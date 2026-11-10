use std::sync::Arc;

use amm_client::link::ManagerLink;
use amm_core::errors::LinkError;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use amml_i18n::localize::Localizer;
use minimap_core::config::{Config, ConfigClamp};
use minimap_core::structs::draw::TextureGrant;

use crate::services::TextureGrantReader;
use crate::traits::ManagerPort;

impl ManagerPort for ManagerLink {
    fn drain_events(&mut self, out: &mut Vec<ModEvent>) {
        loop {
            let before = out.len();
            out.extend(self.events());
            if out.len() == before {
                return;
            }
        }
    }

    fn config(&self) -> Result<Config, LinkError> {
        ManagerLink::config::<Config>(self).map(ConfigClamp::apply)
    }

    fn acquire(&self, capability: Capability) -> Result<AcquireOutcome, LinkError> {
        ManagerLink::acquire(self, capability)
    }

    fn release(&self, capability: Capability) {
        ManagerLink::release(self, capability);
    }

    fn set_setting(&self, path: &str, value: &hcl::Value) -> Result<(), LinkError> {
        ManagerLink::set_setting(self, path, value)
    }

    fn set_cursor_mode(&self, on: bool) -> Result<(), LinkError> {
        ManagerLink::set_cursor_mode(self, on)
    }

    fn holder(&self, capability: Capability) -> Option<String> {
        ManagerLink::holder(self, capability)
    }

    fn texture_budget_grant(&self) -> TextureGrant {
        TextureGrantReader::read(ManagerLink::texture_budget(self))
    }

    fn locale(&self) -> Option<String> {
        ManagerLink::locale(self)
    }

    fn set_localizer(&self, localizer: Arc<Localizer>) {
        ManagerLink::set_localizer(self, localizer);
    }
}
