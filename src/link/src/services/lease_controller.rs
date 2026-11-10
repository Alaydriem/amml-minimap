use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use minimap_core::structs::view::ViewFlags;

use crate::structs::{LeaseEffect, LeaseState};
use crate::traits::ManagerPort;

// keeps the manager leases in step with the view: Hud while the mod is enabled, Overlay and Input with the
// cursor while the world map is open; it calls the port only when a lease has to change
pub struct LeaseController {
    enabled: bool,
    hud: LeaseState,
    overlay: LeaseState,
    input: LeaseState,
    cursor_on: bool,
}

impl LeaseController {
    pub fn new() -> Self {
        Self {
            enabled: true,
            hud: LeaseState::Released,
            overlay: LeaseState::Released,
            input: LeaseState::Released,
            cursor_on: false,
        }
    }

    pub fn on_event(&mut self, event: ModEvent) -> Option<LeaseEffect> {
        match event {
            ModEvent::Disabled => {
                // the manager has already taken every lease back
                self.enabled = false;
                self.hud = LeaseState::Released;
                self.overlay = LeaseState::Released;
                self.input = LeaseState::Released;
                self.cursor_on = false;
                Some(LeaseEffect::Pause)
            }
            ModEvent::Enabled => {
                self.enabled = true;
                Some(LeaseEffect::Resume)
            }
            ModEvent::Granted(capability) | ModEvent::Resumed(capability) => {
                if let Some(state) = self.state(capability) {
                    *state = LeaseState::Held;
                }
                None
            }
            ModEvent::Suspended { capability, .. } => self.lost(capability, LeaseState::Suspended),
            ModEvent::Revoked { capability, .. } => self.lost(capability, LeaseState::Released),
            ModEvent::CapabilityAvailable(capability) => {
                if let Some(state) = self.state(capability)
                    && *state == LeaseState::Refused
                {
                    *state = LeaseState::Released;
                }
                None
            }
            _ => None,
        }
    }

    pub fn sync<P: ManagerPort>(&mut self, port: &P, flags: ViewFlags) -> Option<LeaseEffect> {
        if !self.enabled {
            return None;
        }
        if self.hud == LeaseState::Released {
            self.hud = Self::take(port, Capability::Hud);
        }
        if flags.world_map_open { self.open(port) } else { self.close(port) }
    }

    // gives every lease back, the cursor first; used at shutdown
    pub fn release_all<P: ManagerPort>(&mut self, port: &P) {
        self.close(port);
        if self.hud.is_taken() {
            port.release(Capability::Hud);
        }
        self.hud = LeaseState::Released;
    }

    fn open<P: ManagerPort>(&mut self, port: &P) -> Option<LeaseEffect> {
        if self.overlay == LeaseState::Released {
            self.overlay = Self::take(port, Capability::Overlay);
            if self.overlay == LeaseState::Refused {
                self.overlay = LeaseState::Released;
                return Some(LeaseEffect::CloseWorldMap);
            }
        }
        if self.input == LeaseState::Released {
            self.input = Self::take(port, Capability::Input);
            if self.input == LeaseState::Refused {
                self.input = LeaseState::Released;
                port.release(Capability::Overlay);
                self.overlay = LeaseState::Released;
                return Some(LeaseEffect::CloseWorldMap);
            }
        }
        if !self.cursor_on && self.overlay == LeaseState::Held && self.input == LeaseState::Held {
            Self::cursor(port, true);
            self.cursor_on = true;
        }
        None
    }

    fn close<P: ManagerPort>(&mut self, port: &P) -> Option<LeaseEffect> {
        if self.cursor_on {
            Self::cursor(port, false);
            self.cursor_on = false;
        }
        for capability in [Capability::Input, Capability::Overlay] {
            if let Some(state) = self.state(capability) {
                if state.is_taken() {
                    port.release(capability);
                }
                *state = LeaseState::Released;
            }
        }
        None
    }

    // a lease the world map needs was taken away: the map closes, and the next sync turns the cursor off
    fn lost(&mut self, capability: Capability, state: LeaseState) -> Option<LeaseEffect> {
        let map_lease = matches!(capability, Capability::Overlay | Capability::Input);
        let slot = self.state(capability)?;
        let was_held = *slot == LeaseState::Held;
        *slot = state;
        (map_lease && was_held).then_some(LeaseEffect::CloseWorldMap)
    }

    fn take<P: ManagerPort>(port: &P, capability: Capability) -> LeaseState {
        match port.acquire(capability) {
            Ok(AcquireOutcome::Granted) => LeaseState::Held,
            Ok(AcquireOutcome::Denied { .. } | AcquireOutcome::Preempted) => {
                let holder = port.holder(capability).unwrap_or_default();
                curia::info!("lease refused", { capability: format!("{capability:?}"), holder: holder });
                LeaseState::Refused
            }
            Err(error) => {
                curia::warn!("lease request failed", { capability: format!("{capability:?}"), error: error.to_string() });
                LeaseState::Refused
            }
        }
    }

    fn cursor<P: ManagerPort>(port: &P, on: bool) {
        if let Err(error) = port.set_cursor_mode(on) {
            curia::warn!("cursor mode not changed", { on: on, error: error.to_string() });
        }
    }

    // the lease slot of a capability minimap claims; None for one it does not claim
    fn state(&mut self, capability: Capability) -> Option<&mut LeaseState> {
        match capability {
            Capability::Hud => Some(&mut self.hud),
            Capability::Overlay => Some(&mut self.overlay),
            Capability::Input => Some(&mut self.input),
            _ => None,
        }
    }
}

impl Default for LeaseController {
    fn default() -> Self {
        Self::new()
    }
}
