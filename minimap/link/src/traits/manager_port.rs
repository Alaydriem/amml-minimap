use std::sync::Arc;

use amm_core::errors::LinkError;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use amml_i18n::localize::Localizer;
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;

// the part of the manager link that minimap's game-thread services call; ManagerLink in the mod, a fake in tests
pub trait ManagerPort {
    // every event queued for the mod, in order, appended to `out`
    fn drain_events(&mut self, out: &mut Vec<ModEvent>);

    // the manager's checked config.hcl, with the cross-field rules applied
    fn config(&self) -> Result<Config, LinkError>;

    fn acquire(&self, capability: Capability) -> Result<AcquireOutcome, LinkError>;

    fn release(&self, capability: Capability);

    fn set_setting(&self, path: &str, value: &hcl::Value) -> Result<(), LinkError>;

    fn set_cursor_mode(&self, on: bool) -> Result<(), LinkError>;

    // the id of the mod holding `capability`; it allocates, so it is called only to name a refusal
    fn holder(&self, capability: Capability) -> Option<String>;

    // the texture budget the manager granted; an older manager's fixed grant when it cannot say
    fn texture_budget_grant(&self) -> TextureGrant;

    // the language code the person picked in the manager; None on a manager older than ABI 1.5
    fn locale(&self) -> Option<String>;

    // the localizer the world map's panels resolve their words with, from the next frame on
    fn set_localizer(&self, localizer: Arc<Localizer>);
}
