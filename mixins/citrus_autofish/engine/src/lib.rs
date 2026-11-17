use horizon_client::mod_api::{
    ClientboundPlayPacket, ClientPacketEvent, CommandEvent, CommandsEvent, Flow, Hooks, Host,
    InputKeyEvent, ModsEvent, NativeMod, PlayerFishEvent,
    ScreenKeyPressedPreEvent, ScreenMouseButtonPressedPreEvent, ScreenRenderPostEvent,
    ScreenTextFieldCharTypedPreEvent, ServerboundPlayPacket, SettingsClickEvent, SettingsEvent,
    TickEvent, winit,
};
use std::sync::{
    Mutex,
    atomic::{AtomicBool, AtomicI8, AtomicU8, AtomicU32, Ordering},
};

const MOD_ID: &str = "citrus_autofish";
const DEFAULT_RECAST_DELAY_TICKS: u8 = 4;
/// Serverbound packet id in Horizon revision 9f42064.
const USE_ITEM_PACKET_ID: i32 = 67;
/// Serverbound SetCarriedItem packet id in Horizon revision 9f42064.
const SET_CARRIED_ITEM_PACKET_ID: i32 = 54;
const NO_SLOT: i8 = -1;
const ROD_MAX_DAMAGE: u32 = 64;
const DEFAULT_BREAK_REMAINING: u32 = 8;

fn append_varint(mut value: u32, bytes: &mut Vec<u8>) {
    while value & !0x7f != 0 {
        bytes.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

#[derive(Clone, Copy, Default)]
struct HotbarStack {
    fishing_rod: bool,
    damage: u32,
}

/// State shared by the local server's fishing hooks and the client's tick hook.
#[derive(Default)]
struct AutoFish {
    enabled: AtomicBool,
    reel_pending: AtomicBool,
    recast_ticks: AtomicU8,
    /// Out-of-band sequence numbers for the direct `UseItem` packet path.
    packet_sequence: AtomicU32,
    /// A best-effort view of the nine hotbar stacks, populated from server packets.
    hotbar: Mutex<[HotbarStack; 9]>,
    selected_slot: AtomicU8,
    previous_slot: AtomicI8,
    queued_slot: AtomicI8,
    /// A slot change was sent; wait for the server's SetHeldSlot acknowledgement.
    slot_change_in_flight: AtomicBool,
    restore_after_catch: AtomicBool,
    settings_loaded: AtomicBool,
    open_menu_pending: AtomicBool,
    clear_dot_input: AtomicBool,
}

impl AutoFish {
    fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        if !enabled {
            self.reel_pending.store(false, Ordering::Release);
            self.recast_ticks.store(0, Ordering::Release);
        }
    }

    fn next_packet_sequence(&self) -> i32 {
        (self.packet_sequence.fetch_add(1, Ordering::AcqRel) % i32::MAX as u32 + 1) as i32
    }

    fn use_rod(&self, host: &mut dyn Host) {
        let Some(player) = host.player() else {
            return;
        };
        let mut payload = Vec::with_capacity(11);
        append_varint(0, &mut payload); // main hand
        append_varint(self.next_packet_sequence() as u32, &mut payload);
        payload.extend_from_slice(&player.yaw.to_be_bytes());
        payload.extend_from_slice(&player.pitch.to_be_bytes());
        if let Ok(packet) = ServerboundPlayPacket::decode(USE_ITEM_PACKET_ID, &payload) {
            let _ = host.send_packet(packet);
        }
    }

    fn config_bool(host: &dyn Host, key: &str, default: bool) -> bool {
        host.setting(MOD_ID, key)
            .and_then(|value| value.parse::<bool>().ok())
            .unwrap_or(default)
    }

    fn break_threshold(host: &dyn Host) -> u32 {
        host.setting(MOD_ID, "break_remaining")
            .and_then(|value| value.parse::<u32>().ok())
            .filter(|value| (1..=32).contains(value))
            .unwrap_or(DEFAULT_BREAK_REMAINING)
    }

    fn recast_delay(host: &dyn Host) -> u8 {
        host.setting(MOD_ID, "recast_delay")
            .and_then(|value| value.parse::<u8>().ok())
            .filter(|value| [2, 4, 6, 8].contains(value))
            .unwrap_or(DEFAULT_RECAST_DELAY_TICKS)
    }

    fn cycle_threshold(current: u32) -> u32 {
        match current {
            1 => 2,
            2 => 4,
            4 => 8,
            8 => 16,
            16 => 32,
            _ => 1,
        }
    }

    fn cycle_recast_delay(current: u8) -> u8 {
        match current {
            2 => 4,
            4 => 6,
            6 => 8,
            _ => 2,
        }
    }

    fn save_setting(host: &dyn Host, key: &str, value: &str) {
        host.set_setting(MOD_ID, key, value);
    }

    fn open_menu(&self, host: &mut dyn Host) -> bool {
        host.mod_screen_open(MOD_ID)
    }

    fn set_menu_enabled(&self, host: &dyn Host) {
        self.set_menu_enabled_to(host, !self.enabled.load(Ordering::Acquire));
    }

    fn set_menu_enabled_to(&self, host: &dyn Host, enabled: bool) {
        self.set_enabled(enabled);
        Self::save_setting(host, "enabled", if enabled { "true" } else { "false" });
    }

    /// Run an owned command, deferring its menu until the chat screen has closed.
    fn run_command(&self, host: &dyn Host, command: &str) -> bool {
        let mut words = command.split_whitespace();
        let Some(name) = words.next().map(|name| name.trim_start_matches('/')) else {
            return false;
        };
        if !matches!(name, "autofish" | ".autofish") {
            return false;
        }
        match words.next() {
            None | Some("toggle") => self.set_menu_enabled(host),
            Some("on") => self.set_menu_enabled_to(host, true),
            Some("off") => self.set_menu_enabled_to(host, false),
            Some("menu" | "settings" | "help") => {
                self.open_menu_pending.store(true, Ordering::Release);
            }
            _ => return false,
        }
        true
    }

    fn queue_slot(&self, slot: u8) {
        self.queued_slot.store(slot as i8, Ordering::Release);
    }

    fn send_queued_slot(&self, host: &mut dyn Host) -> bool {
        if self.slot_change_in_flight.load(Ordering::Acquire) {
            return true;
        }
        let slot = self.queued_slot.load(Ordering::Acquire);
        if !(0..=8).contains(&slot) {
            return false;
        }
        let mut payload = Vec::with_capacity(2);
        payload.extend_from_slice(&(slot as i16).to_be_bytes());
        match ServerboundPlayPacket::decode(SET_CARRIED_ITEM_PACKET_ID, &payload) {
            Ok(packet) if host.send_packet(packet).is_ok() => {
                self.slot_change_in_flight.store(true, Ordering::Release);
                true
            }
            _ => false,
        }
    }

    fn observe_packet(&self, host: &dyn Host, packet: &ClientboundPlayPacket) {
        let mut hotbar = self.hotbar.lock().unwrap_or_else(|poison| poison.into_inner());
        let mut update = |slot: usize, stack| {
            if slot >= hotbar.len() {
                return;
            }
            hotbar[slot] = match stack {
                Some(stack) => {
                    let fishing_rod = host
                        .registry_name("minecraft:item", stack.item)
                        .is_some_and(|name| name.ends_with(":fishing_rod"));
                    HotbarStack {
                        fishing_rod,
                        damage: stack.damage().max(0) as u32,
                    }
                }
                None => HotbarStack::default(),
            };
        };
        match packet {
            ClientboundPlayPacket::SetHeldSlot(packet) if (0..=8).contains(&packet.slot) => {
                self.selected_slot.store(packet.slot as u8, Ordering::Release);
                let queued = self.queued_slot.load(Ordering::Acquire);
                if queued == packet.slot as i8 {
                    self.queued_slot.store(NO_SLOT, Ordering::Release);
                    self.slot_change_in_flight.store(false, Ordering::Release);
                }
            }
            ClientboundPlayPacket::ContainerSetSlot(packet) if packet.container == 0 => {
                if (36..=44).contains(&packet.slot) {
                    update((packet.slot - 36) as usize, packet.item.as_ref());
                }
            }
            ClientboundPlayPacket::ContainerSetContent(packet) if packet.container == 0 => {
                for (index, stack) in packet.items.iter().enumerate() {
                    if (36..=44).contains(&index) {
                        update(index - 36, stack.as_ref());
                    }
                }
            }
            ClientboundPlayPacket::SetPlayerInventory(packet) if (0..=8).contains(&packet.slot) => {
                update(packet.slot as usize, packet.item.as_ref());
            }
            _ => {}
        }
    }

    fn maybe_queue_rod_swap(&self, host: &dyn Host) {
        if !Self::config_bool(host, "auto_rod", true)
            || self.slot_change_in_flight.load(Ordering::Acquire)
            || self.queued_slot.load(Ordering::Acquire) != NO_SLOT
        {
            return;
        }
        let selected = self.selected_slot.load(Ordering::Acquire) as usize;
        if selected >= 9 {
            return;
        }
        let threshold = Self::break_threshold(host);
        let hotbar = self.hotbar.lock().unwrap_or_else(|poison| poison.into_inner());
        let current = hotbar[selected];
        let remaining = ROD_MAX_DAMAGE.saturating_sub(current.damage);
        if !current.fishing_rod || remaining > threshold {
            return;
        }
        let replacement = hotbar
            .iter()
            .enumerate()
            .filter(|(slot, stack)| *slot != selected && stack.fishing_rod)
            .filter(|(_, stack)| ROD_MAX_DAMAGE.saturating_sub(stack.damage) > threshold)
            .max_by_key(|(_, stack)| ROD_MAX_DAMAGE.saturating_sub(stack.damage))
            .map(|(slot, _)| slot);
        if let Some(slot) = replacement {
            self.previous_slot.store(selected as i8, Ordering::Release);
            self.queue_slot(slot as u8);
        } else if self.enabled.load(Ordering::Acquire)
            && Self::config_bool(host, "stop_no_spare", true)
        {
            // Protect a critical rod when no usable backup is available.
            self.set_enabled(false);
            Self::save_setting(host, "enabled", "false");
        }
    }

    fn maybe_restore_rod(&self, host: &mut dyn Host) {
        if !self.restore_after_catch.swap(false, Ordering::AcqRel)
            || !Self::config_bool(host, "return_after_catch", true)
        {
            return;
        }
        let previous = self.previous_slot.load(Ordering::Acquire);
        if !(0..=8).contains(&previous) {
            return;
        }
        let selected = self.selected_slot.load(Ordering::Acquire) as usize;
        if selected == previous as usize || self.slot_change_in_flight.load(Ordering::Acquire) {
            return;
        }
        let threshold = Self::break_threshold(host);
        let hotbar = self.hotbar.lock().unwrap_or_else(|poison| poison.into_inner());
        let old = hotbar[previous as usize];
        // Do not return to a rod that is still in the danger zone.
        if old.fishing_rod && ROD_MAX_DAMAGE.saturating_sub(old.damage) > threshold {
            self.queue_slot(previous as u8);
        }
    }
}

impl Hooks for AutoFish {
    /// Slash command aliases are handled locally.
    fn command(&self, host: &mut dyn Host, event: &mut CommandEvent<'_>) -> Flow {
        if self.run_command(host, event.command) {
            Flow::Cancel
        } else {
            Flow::Continue
        }
    }

    /// Also accept dot-prefixed commands typed as ordinary chat, without `/`.
    fn screen_key_pressed_pre(
        &self,
        host: &dyn Host,
        event: &mut ScreenKeyPressedPreEvent<'_>,
    ) -> Flow {
        if event.screen.id != "chat"
            || event.repeat
            || !matches!(event.key, "key.keyboard.enter" | "key.keyboard.keypad.enter")
        {
            return Flow::Continue;
        }
        let Some((input, _)) = host.chat_input() else {
            return Flow::Continue;
        };
        let input = input.trim();
        if !input.starts_with('.') || !self.run_command(host, input) {
            return Flow::Continue;
        }
        self.clear_dot_input.store(true, Ordering::Release);
        Flow::Cancel
    }

    /// When the exact dot command is typed, turn it into a local slash command.
    fn screen_text_field_char_typed_pre(
        &self,
        _host: &dyn Host,
        event: &mut ScreenTextFieldCharTypedPreEvent<'_, '_>,
    ) -> Flow {
        let field = &mut event.field;
        if event.screen.id == "chat"
            && event.character == 'h'
            && field.value == ".autofis"
            && field.cursor == field.value.chars().count()
            && field.highlight == field.cursor
        {
            field.value = "/.autofish".to_owned();
            field.cursor = field.value.chars().count();
            field.highlight = field.cursor;
            return Flow::Cancel;
        }
        Flow::Continue
    }

    /// Publish both local command aliases so Horizon completes them in chat.
    fn commands(&self, _host: &mut dyn Host, event: &mut CommandsEvent<'_, '_>) {
        event.commands.push("autofish");
        event.commands.push(".autofish");
    }

    /// Right Shift is a second shortcut for the settings screen.
    fn input_key(&self, host: &mut dyn Host, event: &mut InputKeyEvent<'_>) {
        if self.clear_dot_input.swap(false, Ordering::AcqRel) {
            let _ = host.set_chat_input("", 0);
        }
        let key = event.event;
        if key.state == winit::event::ElementState::Pressed
            && !key.repeat
            && key.physical_key == winit::keyboard::PhysicalKey::Code(winit::keyboard::KeyCode::ShiftRight)
        {
            if host.mod_screen() == Some(MOD_ID) {
                host.mod_screen_close(MOD_ID);
            } else {
                let _ = self.open_menu(host);
            }
        }
    }

    /// Render the compact AutoFish settings screen over the world.
    fn screen_render_post(&self, host: &dyn Host, event: &mut ScreenRenderPostEvent<'_, '_>) {
        let screen = event.screen;
        if screen.id != "mod" || host.mod_screen() != Some(MOD_ID) {
            return;
        }
        let [_, _, width, height] = screen.rect;
        let panel_w = 270.0_f32.min(width - 24.0).max(220.0);
        let panel_h = 242.0_f32;
        let x = (width - panel_w) * 0.5;
        let y = (height - panel_h) * 0.5;
        host.fill(event.batch, [x, y, panel_w, panel_h], 0xee10151f);
        host.fill(event.batch, [x, y, panel_w, 22.0], 0xff26354a);
        host.text(event.batch, "Citrus AutoFish", [x + 10.0, y + 7.0], 0xffffffff, true);

        let rows = [
            ("AutoFish", Self::config_bool(host, "enabled", self.enabled.load(Ordering::Acquire))),
            ("Auto-switch rod", Self::config_bool(host, "auto_rod", true)),
            ("Return after catch", Self::config_bool(host, "return_after_catch", true)),
            ("Stop if no safe spare rod", Self::config_bool(host, "stop_no_spare", true)),
        ];
        for (index, (label, on)) in rows.into_iter().enumerate() {
            let row_y = y + 36.0 + index as f32 * 29.0;
            host.fill(event.batch, [x + 10.0, row_y - 4.0, panel_w - 20.0, 23.0], 0xff202a39);
            host.text(event.batch, label, [x + 18.0, row_y + 2.0], 0xffeeeeee, true);
            host.text(
                event.batch,
                if on { "ON" } else { "OFF" },
                [x + panel_w - 48.0, row_y + 2.0],
                if on { 0xff67e8a5 } else { 0xffff8585 },
                true,
            );
        }
        let threshold_y = y + 155.0;
        host.fill(event.batch, [x + 10.0, threshold_y - 4.0, panel_w - 20.0, 23.0], 0xff202a39);
        let threshold = Self::break_threshold(host);
        host.text(event.batch, "Swap with durability left", [x + 18.0, threshold_y + 2.0], 0xffeeeeee, true);
        host.text(event.batch, &format!("{threshold}"), [x + panel_w - 34.0, threshold_y + 2.0], 0xffffffff, true);
        let delay_y = y + 184.0;
        host.fill(event.batch, [x + 10.0, delay_y - 4.0, panel_w - 20.0, 23.0], 0xff202a39);
        host.text(event.batch, "Recast delay (ticks)", [x + 18.0, delay_y + 2.0], 0xffeeeeee, true);
        host.text(event.batch, &format!("{}", Self::recast_delay(host)), [x + panel_w - 34.0, delay_y + 2.0], 0xffffffff, true);
        host.text(event.batch, "Click a row to change it · Right Shift / Esc to close", [x + 10.0, y + 218.0], 0xffb8c0cd, false);
    }

    /// The settings screen owns its clicks; button rows toggle their setting.
    fn screen_mouse_button_pressed_pre(
        &self,
        host: &dyn Host,
        event: &mut ScreenMouseButtonPressedPreEvent<'_>,
    ) -> Flow {
        if event.button != 0 || event.screen.id != "mod" || host.mod_screen() != Some(MOD_ID) {
            return Flow::Continue;
        }
        let [_, _, width, height] = event.screen.rect;
        let panel_w = 270.0_f32.min(width - 24.0).max(220.0);
        let panel_h = 242.0_f32;
        let x = (width - panel_w) * 0.5;
        let y = (height - panel_h) * 0.5;
        let [mouse_x, mouse_y] = event.screen.mouse;
        if mouse_x < x + 8.0 || mouse_x > x + panel_w - 8.0 {
            return Flow::Continue;
        }
        let row = ((mouse_y - (y + 32.0)) / 29.0).floor() as i32;
        match row {
            0 => self.set_menu_enabled(host),
            1 => {
                let value = !Self::config_bool(host, "auto_rod", true);
                Self::save_setting(host, "auto_rod", if value { "true" } else { "false" });
            }
            2 => {
                let value = !Self::config_bool(host, "return_after_catch", true);
                Self::save_setting(host, "return_after_catch", if value { "true" } else { "false" });
            }
            3 => {
                let value = !Self::config_bool(host, "stop_no_spare", true);
                Self::save_setting(host, "stop_no_spare", if value { "true" } else { "false" });
            }
            4 => {
                let next = Self::cycle_threshold(Self::break_threshold(host));
                Self::save_setting(host, "break_remaining", &next.to_string());
            }
            5 => {
                let next = Self::cycle_recast_delay(Self::recast_delay(host));
                Self::save_setting(host, "recast_delay", &next.to_string());
            }
            _ => return Flow::Continue,
        }
        Flow::Cancel
    }

    /// Register the mod so Horizon exposes its standard native-mod settings page.
    fn mods(&self, _host: &mut dyn Host, event: &mut ModsEvent<'_, '_>) {
        event.mods.push(NativeMod {
            id: MOD_ID.to_owned(),
            title: "Citrus AutoFish".to_owned(),
            description: "Auto fishing and hotbar rod protection".to_owned(),
            keys: Vec::new(),
        });
    }

    /// The Mods page also exposes these settings in Horizon's standard native-mod menu.
    fn settings(&self, host: &mut dyn Host, event: &mut SettingsEvent<'_, '_>) {
        if event.mod_id != MOD_ID {
            return;
        }
        event.buttons.push(format!(
            "AutoFish: {}",
            if Self::config_bool(host, "enabled", self.enabled.load(Ordering::Acquire)) { "ON" } else { "OFF" }
        ));
        event.buttons.push(format!(
            "Auto-switch rod: {}",
            if Self::config_bool(host, "auto_rod", true) { "ON" } else { "OFF" }
        ));
        event.buttons.push(format!(
            "Return after catch: {}",
            if Self::config_bool(host, "return_after_catch", true) { "ON" } else { "OFF" }
        ));
        event.buttons.push(format!(
            "Stop without spare rod: {}",
            if Self::config_bool(host, "stop_no_spare", true) { "ON" } else { "OFF" }
        ));
        event.buttons.push(format!("Swap at {} durability", Self::break_threshold(host)));
        event.buttons.push(format!("Recast delay: {} ticks", Self::recast_delay(host)));
    }

    fn settings_click(&self, host: &mut dyn Host, event: &mut SettingsClickEvent<'_>) {
        if event.mod_id != MOD_ID {
            return;
        }
        match event.button {
            0 => self.set_menu_enabled(host),
            1 => {
                let value = !Self::config_bool(host, "auto_rod", true);
                Self::save_setting(host, "auto_rod", if value { "true" } else { "false" });
            }
            2 => {
                let value = !Self::config_bool(host, "return_after_catch", true);
                Self::save_setting(host, "return_after_catch", if value { "true" } else { "false" });
            }
            3 => {
                let value = !Self::config_bool(host, "stop_no_spare", true);
                Self::save_setting(host, "stop_no_spare", if value { "true" } else { "false" });
            }
            4 => {
                let next = Self::cycle_threshold(Self::break_threshold(host));
                Self::save_setting(host, "break_remaining", &next.to_string());
            }
            5 => {
                let next = Self::cycle_recast_delay(Self::recast_delay(host));
                Self::save_setting(host, "recast_delay", &next.to_string());
            }
            _ => {}
        }
    }

    /// The server's fishing events schedule a safe return to the old hotbar slot.
    fn player_fish(&self, host: &mut dyn Host, event: &mut PlayerFishEvent<'_>) -> Flow {
        if !self.enabled.load(Ordering::Acquire) {
            return Flow::Continue;
        }
        match event.state {
            // FishingHook.catchingFish: a fish is biting now.
            1 => {
                self.recast_ticks.store(0, Ordering::Release);
                self.reel_pending.store(true, Ordering::Release);
            }
            // The bobber was reeled in: catch, empty line, or hooked entity.
            2..=4 => {
                self.reel_pending.store(false, Ordering::Release);
                self.recast_ticks
                    .store(Self::recast_delay(host), Ordering::Release);
                self.restore_after_catch.store(true, Ordering::Release);
            }
            _ => {}
        }
        Flow::Continue
    }

    /// Keep local hotbar durability and the server-selected slot in sync.
    fn client_packet(&self, host: &mut dyn Host, event: &mut ClientPacketEvent<'_>) {
        self.observe_packet(host, event.packet);
    }

    fn tick(&self, host: &mut dyn Host, _event: &mut TickEvent) {
        if !host.window() || !host.playing() {
            return;
        }
        if !self.settings_loaded.swap(true, Ordering::AcqRel) {
            self.set_enabled(Self::config_bool(host, "enabled", false));
        }
        if self.open_menu_pending.load(Ordering::Acquire) && self.open_menu(host) {
            self.open_menu_pending.store(false, Ordering::Release);
            return;
        }
        self.maybe_restore_rod(host);
        if self.send_queued_slot(host) {
            return;
        }
        self.maybe_queue_rod_swap(host);
        if self.send_queued_slot(host) {
            return;
        }
        if !self.enabled.load(Ordering::Acquire) {
            return;
        }
        if self.reel_pending.swap(false, Ordering::AcqRel) {
            self.use_rod(host);
            return;
        }
        let mut remaining = self.recast_ticks.load(Ordering::Acquire);
        while remaining > 0 {
            let next = remaining - 1;
            match self.recast_ticks.compare_exchange_weak(
                remaining,
                next,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => {
                    if next == 0 && self.enabled.load(Ordering::Acquire) {
                        self.use_rod(host);
                    }
                    return;
                }
                Err(actual) => remaining = actual,
            }
        }
    }
}

/// Entry point used by Horizon's native-mod loader.
pub fn hooks() -> Box<dyn Hooks> {
    Box::new(AutoFish::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_disabled_and_disabling_clears_queued_actions() {
        let fish = AutoFish::default();
        assert!(!fish.enabled.load(Ordering::Acquire));
        fish.set_enabled(true);
        fish.reel_pending.store(true, Ordering::Release);
        fish.recast_ticks.store(3, Ordering::Release);
        fish.set_enabled(false);
        assert!(!fish.enabled.load(Ordering::Acquire));
        assert!(!fish.reel_pending.load(Ordering::Acquire));
        assert_eq!(fish.recast_ticks.load(Ordering::Acquire), 0);
    }

    #[test]
    fn packet_sequence_stays_non_negative_and_starts_at_one() {
        let fish = AutoFish::default();
        assert_eq!(fish.next_packet_sequence(), 1);
        assert_eq!(fish.next_packet_sequence(), 2);
    }

    #[test]
    fn varint_packet_fields_use_minecraft_wire_encoding() {
        let mut bytes = Vec::new();
        append_varint(0, &mut bytes);
        append_varint(127, &mut bytes);
        append_varint(128, &mut bytes);
        assert_eq!(bytes, [0x00, 0x7f, 0x80, 0x01]);
    }
}
