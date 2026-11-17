use horizon_client::mod_api::{
    self, ClientHost, ClientTickPostEvent, CommandEvent, Flow, Hooks, PlayerFishEvent, ServerHost,
    Text,
};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};

const RECAST_DELAY_TICKS: u8 = 4;

/// State shared by the local server's fishing hooks and the client's input queue.
#[derive(Default)]
struct AutoFish {
    enabled: AtomicBool,
    reel_pending: AtomicBool,
    recast_ticks: AtomicU8,
}

impl AutoFish {
    fn set_enabled(&self, enabled: bool) {
        self.enabled.store(enabled, Ordering::Release);
        if !enabled {
            self.reel_pending.store(false, Ordering::Release);
            self.recast_ticks.store(0, Ordering::Release);
        }
    }
}

impl Hooks for AutoFish {
    /// `/autofish` is handled locally and never sent as a server command.
    fn command(&self, host: &mut dyn ClientHost, event: &mut CommandEvent<'_>) -> Flow {
        if event.command.trim() != "autofish" {
            return Flow::Continue;
        }

        let enabled = !self.enabled.load(Ordering::Acquire);
        self.set_enabled(enabled);
        let status = if enabled { "включена" } else { "выключена" };
        let message = Text::literal(format!("Citrus AutoFish: авто-рыбалка {status}"));
        let _ = host.chat_line(message);
        event.ran = true;
        Flow::Cancel
    }

    /// A server-side bite schedules one ordinary use-item click on the client.
    fn player_fish(&self, _host: &mut dyn ServerHost, event: &mut PlayerFishEvent<'_>) -> Flow {
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

    /// Use Horizon's normal input queue; regular item, target, and server checks still apply.
    fn client_tick_post(&self, host: &mut dyn ClientHost, _event: &mut ClientTickPostEvent) {
        if !host.playing() || !self.enabled.load(Ordering::Acquire) {
            return;
        }

        if self.reel_pending.swap(false, Ordering::AcqRel) {
            let _ = host.click_mapping("key.use");
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
                        let _ = host.click_mapping("key.use");
                    }
                    return;
                }
                Err(actual) => remaining = actual,
            }
        }
    }
}

/// Entry point used by Horizon's typed native-mod loader.
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
}
