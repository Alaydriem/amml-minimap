use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::{KeybindPhase, ModEvent};
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use amml_i18n::localize::Localizer;
use amml_i18n::text::Text;
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_link::services::MinimapSession;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{CursorAction, RuntimePaths, ScanPlan, TickInput};
use minimap_store::structs::WorldIdentity;
use minimap_ui::message::MinimapMessage;

use crate::support::{FakeClipboard, FakePort, PortCall};

const TOGGLE_WORLD_MAP: u16 = 0;
const ZOOM_OUT: u16 = 3;

struct World {
    dir: tempfile::TempDir,
}

impl World {
    fn new() -> Self {
        Self { dir: tempfile::tempdir().unwrap() }
    }

    fn runtime(&self, config: Config) -> Arc<Runtime> {
        let paths = RuntimePaths { data_dir: self.dir.path().join("data") };
        let grant = TextureGrant { bytes: 64 * 1024 * 1024, count: 1024 };
        Arc::new(Runtime::new(config, paths, grant).unwrap())
    }

    fn enter(&self, runtime: &Runtime) {
        runtime.enter_world(&WorldIdentity::Local { world_dir: self.dir.path().join("world") }, 0).unwrap();
    }
}

fn session(port: FakePort, runtime: &Arc<Runtime>) -> MinimapSession<FakePort, FakeClipboard> {
    MinimapSession::with_port(port, Arc::clone(runtime), FakeClipboard::default())
}

fn press(action: u16) -> ModEvent {
    ModEvent::Keybind { action, phase: KeybindPhase::Pressed }
}

fn tick_input() -> TickInput {
    let player = PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 };
    TickInput { player, foreground: true, cursor_hidden: true, mouse_grabbed: true, ..TickInput::default() }
}

fn tick(session: &mut MinimapSession<FakePort, FakeClipboard>) -> ScanPlan {
    let mut plan = ScanPlan::default();
    session.tick(&tick_input(), &mut plan);
    plan
}

#[test]
fn keybind_m_opens_the_map_and_takes_overlay_input_and_cursor() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);

    tick(&mut session);

    assert!(runtime.view_flags().world_map_open);
    let calls = calls.borrow();
    for call in [PortCall::Acquire(Capability::Overlay), PortCall::Acquire(Capability::Input), PortCall::Cursor(true)] {
        assert!(calls.contains(&call), "{call:?} in {calls:?}");
    }
    drop(calls);
    session.shutdown();
}

#[test]
fn settings_changed_rereads_config_and_applies_it_live() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.config.mini_map.radius_chunks = 9;
    port.events.push_back(ModEvent::SettingsChanged { paths: 1 });
    port.events.push_back(press(ZOOM_OUT));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);

    tick(&mut session);

    let setting = PortCall::Setting("mini_map.radius_chunks".to_string(), hcl::Value::from(10));
    assert!(calls.borrow().contains(&setting), "{:?}", calls.borrow());
    session.shutdown();
}

#[test]
fn writes_are_flushed_before_the_config_is_read() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.events.push_back(press(ZOOM_OUT));
    port.events.push_back(ModEvent::SettingsChanged { paths: 1 });
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);

    tick(&mut session);

    let calls = calls.borrow();
    let setting = calls.iter().position(|call| matches!(call, PortCall::Setting(..))).expect("a write");
    let config = calls.iter().position(|call| *call == PortCall::Config).expect("a config read");
    assert!(setting < config, "{calls:?}");
    drop(calls);
    session.shutdown();
}

#[test]
fn disabled_stops_scanning_until_enabled() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    world.enter(&runtime);
    let mut port = FakePort::new();
    port.events.push_back(ModEvent::Disabled);
    let mut session = session(port, &runtime);

    let disabled = tick(&mut session);
    session.port_mut().events.push_back(ModEvent::Enabled);
    let enabled = tick(&mut session);

    assert!(disabled.candidates.is_empty());
    assert!(!enabled.candidates.is_empty());
    session.shutdown();
}

#[test]
fn a_denied_overlay_leaves_the_map_closed() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.outcomes.insert(Capability::Overlay, AcquireOutcome::Denied { holder: ModIndex(2) });
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let mut session = session(port, &runtime);

    tick(&mut session);

    assert!(!runtime.view_flags().world_map_open);
    session.shutdown();
}

#[test]
fn a_keybind_in_chat_does_not_open_the_map() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let mut session = session(port, &runtime);
    let mut plan = ScanPlan::default();

    session.tick(&TickInput { mouse_grabbed: false, ..tick_input() }, &mut plan);

    assert!(!runtime.view_flags().world_map_open);
    session.shutdown();
}

#[test]
fn shutdown_releases_every_lease_before_the_runtime_stops() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);
    tick(&mut session);
    calls.borrow_mut().clear();

    session.shutdown();

    assert_eq!(
        *calls.borrow(),
        [
            PortCall::Cursor(false),
            PortCall::Release(Capability::Input),
            PortCall::Release(Capability::Overlay),
            PortCall::Release(Capability::Hud),
        ]
    );
}

// no player; the game can still report a hidden cursor and a grabbed mouse, for example on the world load screen
fn out_of_world() -> TickInput {
    TickInput { player: PlayerSnapshot::default(), ..tick_input() }
}

#[test]
fn leaving_the_world_with_the_map_open_gives_back_input_overlay_and_cursor() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    world.enter(&runtime);
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);
    tick(&mut session);
    calls.borrow_mut().clear();

    runtime.leave_world();
    let mut plan = ScanPlan::default();
    session.tick(&out_of_world(), &mut plan);

    let calls = calls.borrow();
    for call in [PortCall::Cursor(false), PortCall::Release(Capability::Input), PortCall::Release(Capability::Overlay)]
    {
        assert!(calls.contains(&call), "{call:?} in {calls:?}");
    }
    assert_eq!(plan.cursor, CursorAction::Release);
    drop(calls);
    session.shutdown();
}

#[test]
fn a_keybind_pressed_out_of_a_world_opens_nothing_and_takes_no_input() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);
    let mut plan = ScanPlan::default();

    session.tick(&out_of_world(), &mut plan);

    assert!(!runtime.view_flags().world_map_open);
    let calls = calls.borrow();
    assert!(!calls.contains(&PortCall::Acquire(Capability::Input)), "{calls:?}");
    assert!(!calls.contains(&PortCall::Cursor(true)), "{calls:?}");
    drop(calls);
    session.shutdown();
}

#[test]
fn a_setting_changed_in_the_menus_reaches_the_next_world() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let mut port = FakePort::new();
    // two chunks; the default is eight
    port.config.terrain.scan_radius = 32;
    port.events.push_back(ModEvent::SettingsChanged { paths: 1 });
    let mut session = session(port, &runtime);
    let mut plan = ScanPlan::default();

    session.tick(&out_of_world(), &mut plan);
    world.enter(&runtime);
    let first = tick(&mut session);

    let reach = first.candidates.iter().map(|pos| pos.x.abs().max(pos.z.abs())).max().unwrap_or(0);
    assert_eq!(reach, 2, "the new world scans to the radius set in the menus");
    session.shutdown();
}

#[test]
fn disabled_with_the_map_open_closes_it_and_enabled_does_not_open_it_again() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    world.enter(&runtime);
    let mut port = FakePort::new();
    port.events.push_back(press(TOGGLE_WORLD_MAP));
    let calls = Rc::clone(&port.calls);
    let mut session = session(port, &runtime);
    tick(&mut session);

    session.port_mut().events.push_back(ModEvent::Disabled);
    let disabled = tick(&mut session);
    let still_disabled = tick(&mut session);
    session.port_mut().events.push_back(ModEvent::Enabled);
    calls.borrow_mut().clear();
    tick(&mut session);

    assert_eq!(disabled.cursor, CursorAction::Release, "the cursor clip is given back once");
    assert_eq!(still_disabled.cursor, CursorAction::None);
    assert!(!runtime.view_flags().world_map_open);
    let calls = calls.borrow();
    assert!(!calls.contains(&PortCall::Acquire(Capability::Input)), "{calls:?}");
    assert!(!calls.contains(&PortCall::Cursor(true)), "{calls:?}");
    drop(calls);
    session.shutdown();
}

// what the localizer the session last gave the port makes of Save
fn save_word(localizer: &Rc<RefCell<Option<Arc<Localizer>>>>) -> Option<String> {
    let localizer = localizer.borrow();
    localizer.as_ref().map(|localizer| Text::message(MinimapMessage::Save).resolve(localizer).into_owned())
}

#[test]
fn session_start_sets_the_manager_language() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let port = FakePort::new();
    *port.locale.borrow_mut() = Some("en_XA".to_owned());
    let localizer = Rc::clone(&port.localizer);

    let session = session(port, &runtime);

    let word = save_word(&localizer);
    assert!(word.as_deref().is_some_and(|word| word != "Save"), "{word:?}");
    session.shutdown();
}

#[test]
fn locale_changed_sets_the_new_language() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let port = FakePort::new();
    *port.locale.borrow_mut() = Some(String::new());
    let (locale, localizer) = (Rc::clone(&port.locale), Rc::clone(&port.localizer));
    let mut session = session(port, &runtime);
    let english = save_word(&localizer);

    *locale.borrow_mut() = Some("en_XA".to_owned());
    session.port_mut().events.push_back(ModEvent::LocaleChanged);
    tick(&mut session);

    assert_eq!(english.as_deref(), Some("Save"));
    let word = save_word(&localizer);
    assert!(word.as_deref().is_some_and(|word| word != "Save"), "{word:?}");
    session.shutdown();
}

#[test]
fn old_manager_gets_english() {
    let world = World::new();
    let runtime = world.runtime(Config::default());
    let port = FakePort::new();
    let localizer = Rc::clone(&port.localizer);

    let session = session(port, &runtime);

    assert_eq!(save_word(&localizer).as_deref(), Some("Save"));
    session.shutdown();
}
