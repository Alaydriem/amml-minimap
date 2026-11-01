use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;
use std::sync::Arc;

use amm_core::errors::LinkError;
use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use amml_i18n::localize::Localizer;
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_link::traits::ManagerPort;

#[derive(Debug, Clone, PartialEq)]
pub enum PortCall {
    Acquire(Capability),
    Release(Capability),
    Cursor(bool),
    Setting(String, hcl::Value),
    Holder(Capability),
    Config,
}

// records every call in order; acquire answers from a script, granted when nothing is scripted
pub struct FakePort {
    // shared, so a test still reads it after the port is dropped
    pub calls: Rc<RefCell<Vec<PortCall>>>,
    pub outcomes: HashMap<Capability, AcquireOutcome>,
    pub events: VecDeque<ModEvent>,
    pub config: Config,
    pub refuse_settings: bool,
    pub grant: TextureGrant,
    // the language code `locale` answers with (None: a manager older than ABI 1.5), and the last localizer the
    // session gave; shared, so a test changes and reads them after the port moved into the session
    pub locale: Rc<RefCell<Option<String>>>,
    pub localizer: Rc<RefCell<Option<Arc<Localizer>>>>,
}

impl FakePort {
    pub fn new() -> Self {
        Self {
            calls: Rc::new(RefCell::new(Vec::new())),
            outcomes: HashMap::new(),
            events: VecDeque::new(),
            config: Config::default(),
            refuse_settings: false,
            grant: TextureGrant::OLD_MANAGER,
            locale: Rc::new(RefCell::new(None)),
            localizer: Rc::new(RefCell::new(None)),
        }
    }

    pub fn take_calls(&self) -> Vec<PortCall> {
        std::mem::take(&mut *self.calls.borrow_mut())
    }

    fn push(&self, call: PortCall) {
        self.calls.borrow_mut().push(call);
    }
}

impl ManagerPort for FakePort {
    fn drain_events(&mut self, out: &mut Vec<ModEvent>) {
        out.extend(self.events.drain(..));
    }

    fn config(&self) -> Result<Config, LinkError> {
        self.push(PortCall::Config);
        Ok(self.config.clone())
    }

    fn acquire(&self, capability: Capability) -> Result<AcquireOutcome, LinkError> {
        self.push(PortCall::Acquire(capability));
        Ok(self.outcomes.get(&capability).copied().unwrap_or(AcquireOutcome::Granted))
    }

    fn release(&self, capability: Capability) {
        self.push(PortCall::Release(capability));
    }

    fn set_setting(&self, path: &str, value: &hcl::Value) -> Result<(), LinkError> {
        self.push(PortCall::Setting(path.to_string(), value.clone()));
        if self.refuse_settings { Err(LinkError::Status(7)) } else { Ok(()) }
    }

    fn set_cursor_mode(&self, on: bool) -> Result<(), LinkError> {
        self.push(PortCall::Cursor(on));
        Ok(())
    }

    fn holder(&self, capability: Capability) -> Option<String> {
        self.push(PortCall::Holder(capability));
        Some("@alaydriem/demo-b".to_string())
    }

    fn texture_budget_grant(&self) -> TextureGrant {
        self.grant
    }

    fn locale(&self) -> Option<String> {
        self.locale.borrow().clone()
    }

    fn set_localizer(&self, localizer: Arc<Localizer>) {
        *self.localizer.borrow_mut() = Some(localizer);
    }
}
