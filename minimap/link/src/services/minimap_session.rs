use std::path::Path;
use std::sync::{Arc, OnceLock};

use amm_client::link::{ApiTable, ManagerLink};
use amm_core::structs::event::ModEvent;
use minimap_core::structs::input::KeybindInput;
use minimap_core::structs::settings::SettingWrite;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{RuntimePaths, ScanPlan, TickInput};
use minimap_ui::runtime::MinimapLocale;

use crate::errors::SessionError;
use crate::services::{EventRouter, LeaseController, MinimapFrameHandler, SettingsBridge, SystemClipboard};
use crate::structs::{Descriptor, LeaseEffect, RoutedEvent};
use crate::traits::{ClipboardSink, ManagerPort};

// minimap registered with the manager: the game thread's tick drains the manager's events, runs the runtime,
// keeps the leases in step with the view, and saves in-play settings. No runtime lock is held across a port call
pub struct MinimapSession<P: ManagerPort, C: ClipboardSink> {
    port: P,
    runtime: Arc<Runtime>,
    clipboard: C,
    leases: LeaseController,
    // this tick's events and setting writes; both keep their capacity
    events: Vec<ModEvent>,
    writes: Vec<SettingWrite>,
}

impl MinimapSession<ManagerLink, SystemClipboard> {
    // registers with the manager, reads the config it checked and the texture budget it granted, then starts
    // the runtime; `config_dir` is the mod's AMML config folder, an absolute path
    pub fn connect(config_dir: &Path, paths: RuntimePaths) -> Result<Self, SessionError> {
        Self::connect_with(ApiTable::resolve()?, config_dir, paths)
    }

    // as connect, with the manager's table already in hand: the mod's own DLL lookup, or a test host
    pub fn connect_with(table: ApiTable, config_dir: &Path, paths: RuntimePaths) -> Result<Self, SessionError> {
        std::fs::create_dir_all(config_dir)?;
        let slot = Arc::new(OnceLock::new());
        let link = ManagerLink::new(table, Descriptor::TEXT, config_dir, MinimapFrameHandler::new(Arc::clone(&slot)))?;
        let config = ManagerPort::config(&link)?;
        let grant = link.texture_budget_grant();
        let runtime = Arc::new(Runtime::new(config, paths, grant)?);
        let _ = slot.set(Arc::clone(&runtime));
        curia::info!("@alaydriem/minimap registered", { texture_mb: grant.bytes / (1024 * 1024), textures: grant.count });
        Ok(Self::with_port(link, runtime, SystemClipboard))
    }
}

impl<P: ManagerPort, C: ClipboardSink> MinimapSession<P, C> {
    const EVENT_CAPACITY: usize = 64;
    const WRITE_CAPACITY: usize = 8;

    // also gives the world map's panels the manager's language
    pub fn with_port(port: P, runtime: Arc<Runtime>, clipboard: C) -> Self {
        let session = Self {
            port,
            runtime,
            clipboard,
            leases: LeaseController::new(),
            events: Vec::with_capacity(Self::EVENT_CAPACITY),
            writes: Vec::with_capacity(Self::WRITE_CAPACITY),
        };
        session.apply_locale();
        session
    }

    // reads the language the person picked in the manager and gives the world map's panels their words in it; a
    // manager older than ABI 1.5 has no language setting, so the panels show English
    fn apply_locale(&self) {
        let code = self.port.locale().unwrap_or_default();
        self.port.set_localizer(MinimapLocale::localizer(&code));
    }

    pub fn runtime(&self) -> &Arc<Runtime> {
        &self.runtime
    }

    pub fn port_mut(&mut self) -> &mut P {
        &mut self.port
    }

    pub fn tick(&mut self, input: &TickInput, plan: &mut ScanPlan) {
        self.events.clear();
        self.port.drain_events(&mut self.events);
        for index in 0..self.events.len() {
            let event = self.events[index];
            self.route(event, input);
        }
        self.runtime.tick(input, plan);
        if let Some(effect) = self.leases.sync(&self.port, self.runtime.view_flags()) {
            self.apply(effect);
        }
        self.flush_writes();
        if let Some(text) = self.runtime.take_clipboard()
            && let Err(error) = self.clipboard.write(&text)
        {
            curia::warn!("clipboard not written", { error: error });
        }
    }

    // gives every lease back, unregisters, then stops the runtime's threads
    pub fn shutdown(self) {
        let Self { port, runtime, mut leases, .. } = self;
        leases.release_all(&port);
        drop(port);
        runtime.shutdown();
    }

    fn route(&mut self, event: ModEvent, input: &TickInput) {
        match EventRouter::route(event) {
            // a key pressed out of a world, in the menus, is not a keybind
            RoutedEvent::Keybind(_, _) if !input.player.present => {}
            RoutedEvent::Keybind(action, phase) => {
                self.runtime.keybind(KeybindInput { action, phase, mouse_grabbed: input.mouse_grabbed });
            }
            RoutedEvent::SettingsChanged => {
                // an in-play change from earlier in this drain is saved first, so the document read back has it
                self.flush_writes();
                match self.port.config() {
                    Ok(config) => self.runtime.apply_settings(config),
                    Err(error) => curia::warn!("changed settings not read", { error: error.to_string() }),
                }
            }
            RoutedEvent::LocaleChanged => self.apply_locale(),
            RoutedEvent::Lease(event) => {
                if let Some(effect) = self.leases.on_event(event) {
                    self.apply(effect);
                }
            }
            // the world settings are taken when the next world starts
            RoutedEvent::WorldReloadApply | RoutedEvent::Ignored => {}
        }
    }

    fn apply(&self, effect: LeaseEffect) {
        match effect {
            LeaseEffect::CloseWorldMap => self.runtime.close_world_map(),
            LeaseEffect::Pause => {
                self.runtime.close_world_map();
                self.runtime.set_active(false);
            }
            LeaseEffect::Resume => self.runtime.set_active(true),
        }
    }

    fn flush_writes(&mut self) {
        self.writes.clear();
        self.runtime.take_setting_writes(&mut self.writes);
        SettingsBridge::flush(&self.port, &self.writes);
    }
}
