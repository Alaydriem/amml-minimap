use std::sync::{Arc, OnceLock};

use amm_client::link::ManagerLink;
use amm_core::structs::event::ModEvent;
use minimap_core::structs::input::{KeyPhase, KeybindAction, KeybindInput};
use minimap_core::structs::settings::SettingWrite;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{RuntimePaths, ScanPlan, TickInput};
use minimap_ui::runtime::MinimapLocale;

use crate::errors::SessionError;
use crate::services::{BindMatcher, EventRouter, KeyFeed, LeaseController, SettingsBridge, SystemClipboard};
use crate::structs::{LeaseEffect, RoutedEvent};
use crate::traits::{ClipboardSink, ManagerPort};

// minimap registered with the manager: the game thread's tick drains the manager's events, runs the runtime,
// keeps the leases in step with the view, and saves in-play settings. No runtime lock is held across a port call
pub struct MinimapSession<P: ManagerPort, C: ClipboardSink> {
    port: P,
    runtime: Arc<Runtime>,
    clipboard: C,
    // armed each tick while the world map is open and the manager panel is closed
    keys: Arc<KeyFeed>,
    leases: LeaseController,
    // this tick's events, binds fed from the game's keys and setting writes; each keeps its capacity
    events: Vec<ModEvent>,
    binds: Vec<(KeybindAction, KeyPhase)>,
    writes: Vec<SettingWrite>,
}

impl MinimapSession<ManagerLink, SystemClipboard> {
    // starts the runtime on a link that is connected: reads the config the manager checked and the texture budget it
    // granted, then puts the runtime in `slot`, where the frame handler of the link finds it. `keys` is the key feed
    // the frame handler of the link drains
    pub fn open(
        link: ManagerLink,
        slot: &OnceLock<Arc<Runtime>>,
        keys: Arc<KeyFeed>,
        paths: RuntimePaths,
    ) -> Result<Self, SessionError> {
        let config = ManagerPort::config(&link)?;
        keys.set_keys(&config.keys);
        let grant = link.texture_budget_grant();
        let runtime = Arc::new(Runtime::new(config, paths, grant)?);
        let _ = slot.set(Arc::clone(&runtime));
        curia::info!("@alaydriem/minimap registered", { texture_mb: grant.bytes / (1024 * 1024), textures: grant.count });
        Ok(Self::with_port(link, runtime, SystemClipboard, keys))
    }
}

impl<P: ManagerPort, C: ClipboardSink> MinimapSession<P, C> {
    const EVENT_CAPACITY: usize = 64;
    const WRITE_CAPACITY: usize = 8;

    // also gives the world map's panels the manager's language
    pub fn with_port(port: P, runtime: Arc<Runtime>, clipboard: C, keys: Arc<KeyFeed>) -> Self {
        let session = Self {
            port,
            runtime,
            clipboard,
            keys,
            leases: LeaseController::new(),
            events: Vec::with_capacity(Self::EVENT_CAPACITY),
            binds: Vec::with_capacity(BindMatcher::QUEUE_CAPACITY),
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
        self.binds.clear();
        self.keys.take_binds(&mut self.binds);
        for index in 0..self.binds.len() {
            let (action, phase) = self.binds[index];
            self.keybind(action, phase, input);
        }
        self.runtime.tick(input, plan);
        self.keys.set_armed(self.runtime.view_flags().world_map_open && !self.port.panel_open());
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
            RoutedEvent::Keybind(action, phase) => self.keybind(action, phase, input),
            RoutedEvent::SettingsChanged => {
                // an in-play change from earlier in this drain is saved first, so the document read back has it
                self.flush_writes();
                match self.port.config() {
                    Ok(config) => {
                        self.keys.set_keys(&config.keys);
                        self.runtime.apply_settings(config);
                    }
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

    // a key pressed out of a world, in the menus, is not a keybind
    fn keybind(&self, action: KeybindAction, phase: KeyPhase, input: &TickInput) {
        if input.player.present {
            self.runtime.keybind(KeybindInput { action, phase, mouse_grabbed: input.mouse_grabbed });
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
