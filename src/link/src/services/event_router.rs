use amm_core::structs::event::{KeybindPhase, ModEvent};
use minimap_core::structs::input::{KeyPhase, KeybindAction};

use crate::structs::RoutedEvent;

// sorts a manager event into what minimap does with it
pub struct EventRouter;

impl EventRouter {
    pub fn route(event: ModEvent) -> RoutedEvent {
        match event {
            ModEvent::Keybind { action, phase } => match KeybindAction::from_index(action) {
                Some(action) => RoutedEvent::Keybind(action, Self::phase(phase)),
                None => RoutedEvent::Ignored,
            },
            ModEvent::SettingsChanged { .. } => RoutedEvent::SettingsChanged,
            ModEvent::LocaleChanged => RoutedEvent::LocaleChanged,
            ModEvent::WorldReloadApply => RoutedEvent::WorldReloadApply,
            ModEvent::Enabled
            | ModEvent::Disabled
            | ModEvent::Granted(_)
            | ModEvent::Revoked { .. }
            | ModEvent::Suspended { .. }
            | ModEvent::Resumed(_)
            | ModEvent::CapabilityAvailable(_) => RoutedEvent::Lease(event),
            // raw keys and pointer: the world map reads them from the egui input of its frame; and events the
            // manager adds for other mods
            _ => RoutedEvent::Ignored,
        }
    }

    fn phase(phase: KeybindPhase) -> KeyPhase {
        match phase {
            KeybindPhase::Pressed => KeyPhase::Pressed,
            KeybindPhase::Released => KeyPhase::Released,
            KeybindPhase::Repeat => KeyPhase::Repeat,
        }
    }
}
