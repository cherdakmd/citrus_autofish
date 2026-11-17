use horizon_client::mod_api::{CommandEvent, Flow, Hooks, Host, PlayerFishEvent, ServerboundPlayPacket};
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, Ordering};

const RECAST_DELAY_TICKS: u8 = 4;
/// Serverbound packet id in the pinned Horizon revision 9f42064.
const USE_ITEM_PACKET_ID: i32 = 67;

fn append_varint(mut value: u32, bytes: &mut Vec<u8>) {
    while value & !0x7f != 0 {
        bytes.push((value as u8 & 0x7f) | 0x80);
        value >>= 7;
    }
    bytes.push(value as u8);
}

/// State shared by the local server's fishing hooks and the client's tick hook.
#[derive(Default)]
struct AutoFish {
    enabled: AtomicBool,
    reel_pending: AtomicBool,
    recast_ticks: AtomicU8,
    /// Out-of-band sequence numbers for the direct `UseItem` packet path.
    /// Horizon 0.2.69 does not expose its vanilla mapping-click queue.
    packet_sequence: AtomicU32,
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
        // Keep values non-negative (required by the server) while allowing the
        // counter to wrap without overflowing a signed integer.
        (self.packet_sequence.fetch_add(1, Ordering::AcqRel) % i32::MAX as u32 + 1) as i32
    }

    fn use_rod(&self, host: &mut dyn Host) {
        let Some(player) = host.player() else {
            return;
        };
        // This revision re-exports the packet enum but not its `UseItem` payload type.
        // Decode the wire payload through the public enum API to retain Horizon's exact
        // packet type without adding a forbidden direct dependency on horizon-protocol.
        let mut payload = Vec::with_capacity(11);
        append_varint(0, &mut payload); // main hand
        append_varint(self.next_packet_sequence() as u32, &mut payload);
        payload.extend_from_slice(&player.yaw.to_be_bytes());
        payload.extend_from_slice(&player.pitch.to_be_bytes());
        let Ok(packet) = ServerboundPlayPacket::decode(USE_ITEM_PACKET_ID, &payload) else {
            return;
        };
        let _ = host.send_packet(packet);
    }
}

impl Hooks for AutoFish {
    /// `/autofish` is handled locally and never sent as a server command.
    fn command(&self, _host: &mut dyn Host, event: &mut CommandEvent<'_>) -> Flow {
        if event.command.trim() != "autofish" {
            return Flow::Continue;
        }

        let enabled = !self.enabled.load(Ordering::Acquire);
        self.set_enabled(enabled);
        event.ran = true;
        Flow::Cancel
    }

    /// A server-side bite schedules one use-item packet on the client.
    fn player_fish(&self, _host: &mut dyn Host, event: &mut PlayerFishEvent<'_>) -> Flow {
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
                    .store(RECAST_DELAY_TICKS, Ordering::Release);
            }
            _ => {}
        }
        Flow::Continue
    }

    /// Tick hook runs on both client and server Game objects; send only from
    /// the player's active client window.
    fn tick(&self, host: &mut dyn Host) {
        if !host.window() || !host.playing() || !self.enabled.load(Ordering::Acquire) {
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
    fn a_pending_reel_is_consumed_only_once() {
        let fish = AutoFish::default();
        fish.set_enabled(true);
        fish.reel_pending.store(true, Ordering::Release);
        assert!(fish.reel_pending.swap(false, Ordering::AcqRel));
        assert!(!fish.reel_pending.swap(false, Ordering::AcqRel));
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
