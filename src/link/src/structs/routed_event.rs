use amm_core::structs::event::ModEvent;
use minimap_core::structs::input::{KeyPhase, KeybindAction};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RoutedEvent {
    Keybind(KeybindAction, KeyPhase),
    SettingsChanged,
    // the person picked another language in the manager
    LocaleChanged,
    WorldReloadApply,
    // enable, disable and lease changes, for the lease controller
    Lease(ModEvent),
    Ignored,
}
