use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amm_core::structs::registry::ModIndex;
use minimap_core::structs::input::{KeyPhase, KeybindAction};
use minimap_link::services::EventRouter;
use minimap_link::structs::RoutedEvent;

fn pressed(action: u16) -> ModEvent {
    ModEvent::Keybind { action, phase: KeybindPhase::Pressed }
}

#[test]
fn every_descriptor_action_index_maps_to_its_action() {
    for (index, action) in KeybindAction::ALL.iter().enumerate() {
        assert_eq!(EventRouter::route(pressed(index as u16)), RoutedEvent::Keybind(*action, KeyPhase::Pressed));
    }
    assert_eq!(EventRouter::route(pressed(KeybindAction::ALL.len() as u16)), RoutedEvent::Ignored);
}

#[test]
fn keybind_phases_map_one_to_one() {
    let phases = [
        (KeybindPhase::Pressed, KeyPhase::Pressed),
        (KeybindPhase::Released, KeyPhase::Released),
        (KeybindPhase::Repeat, KeyPhase::Repeat),
    ];
    for (manager, minimap) in phases {
        let routed = EventRouter::route(ModEvent::Keybind { action: 2, phase: manager });
        assert_eq!(routed, RoutedEvent::Keybind(KeybindAction::ZoomIn, minimap));
    }
}

#[test]
fn raw_input_events_are_ignored() {
    let raw = [
        ModEvent::Key { vk: 0x1B, down: true, repeat: false },
        ModEvent::PointerButton { button: 0, down: true, x: 1.0, y: 2.0 },
        ModEvent::PointerDelta { dx: 1.0, dy: 1.0 },
        ModEvent::Wheel { delta: 1.0 },
    ];
    for event in raw {
        assert_eq!(EventRouter::route(event), RoutedEvent::Ignored);
    }
}

#[test]
fn menu_entry_clicks_are_ignored() {
    assert_eq!(EventRouter::route(ModEvent::MenuEntryClicked { entry: 0 }), RoutedEvent::Ignored);
}

#[test]
fn settings_and_world_reload_are_routed() {
    assert_eq!(EventRouter::route(ModEvent::SettingsChanged { paths: 3 }), RoutedEvent::SettingsChanged);
    assert_eq!(EventRouter::route(ModEvent::WorldReloadApply), RoutedEvent::WorldReloadApply);
}

#[test]
fn lease_events_pass_through_unchanged() {
    let lease = [
        ModEvent::Enabled,
        ModEvent::Disabled,
        ModEvent::Granted(Capability::Hud),
        ModEvent::Revoked { capability: Capability::Input, by: ModIndex(2) },
        ModEvent::Suspended { capability: Capability::Overlay, by: ModIndex(1) },
        ModEvent::Resumed(Capability::Overlay),
        ModEvent::CapabilityAvailable(Capability::Input),
    ];
    for event in lease {
        assert_eq!(EventRouter::route(event), RoutedEvent::Lease(event));
    }
}
