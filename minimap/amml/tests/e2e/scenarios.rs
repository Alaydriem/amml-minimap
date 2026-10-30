use alaydriem_minimap::services::MapSession;
use alaydriem_minimap::traits::MapSink;
use minimap_core::config::{Config, ConfigClamp};
use minimap_core::structs::view::PlayerSnapshot;
use minimap_runtime::services::CaptureReplay;
use minimap_runtime::structs::{ScanPlan, TickInput};
use minimap_store::structs::WorldIdentity;

use crate::Host;

const M: u32 = 0x4D;
const EQUALS: u32 = 0xBB;
const ESCAPE: u32 = 0x1B;

#[test]
fn the_manager_accepts_the_descriptor_and_writes_the_default_config() {
    let mut host = Host::new();
    for _ in 0..30 {
        host.tick();
    }
    let path = host.config_file();
    // stopping the host flushes its writes
    let _dir = host.shutdown();

    let text = std::fs::read_to_string(path).unwrap();
    let config: Config = hcl::from_str(&text).unwrap();
    assert_eq!(ConfigClamp::apply(config), Config::default());
}

#[test]
fn m_opens_the_world_map_overlay_and_escape_closes_it() {
    let mut host = Host::new();

    host.key(M, 0);
    let open = host.session().runtime().view_flags().world_map_open;
    for _ in 0..3 {
        host.frame();
    }
    host.key(ESCAPE, 0);
    for _ in 0..3 {
        host.frame();
    }
    host.tick();

    assert!(open, "M opened the world map");
    assert!(!host.session().runtime().view_flags().world_map_open, "Escape closed it");
    let _dir = host.shutdown();
}

#[test]
fn equals_changes_the_radius_and_the_manager_saves_it() {
    let mut host = Host::new();

    host.key(EQUALS, 0);
    for _ in 0..30 {
        host.tick();
    }
    let path = host.config_file();
    let _dir = host.shutdown();

    let text = std::fs::read_to_string(path).unwrap();
    assert!(text.contains("radius_chunks = 4"), "{text}");
}

#[test]
fn a_rebound_chord_works_and_the_old_one_does_not() {
    let mut host = Host::new();
    let rebound = host.session().port_mut().set_setting("keys.toggle_world_map", &hcl::Value::from("Ctrl+Shift+M"));
    assert!(rebound.is_ok(), "{rebound:?}");
    for _ in 0..30 {
        host.tick();
    }

    host.key(M, 0);
    let old = host.session().runtime().view_flags().world_map_open;
    host.key(M, Host::CTRL | Host::SHIFT);
    let new = host.session().runtime().view_flags().world_map_open;

    assert!(!old, "plain M no longer opens the map");
    assert!(new, "Ctrl+Shift+M opens it");
    let _dir = host.shutdown();
}

#[test]
fn the_corner_map_draws_on_the_manager_with_the_map_closed() {
    let mut host = Host::new();
    let mut drawn = false;

    for _ in 0..20 {
        host.tick();
        drawn |= !host.frame().is_empty();
    }

    assert!(drawn, "the manager drew a layer of minimap's");
    let _dir = host.shutdown();
}

#[test]
fn every_capture_in_lm_e2e_captures_replays() {
    let Some(dir) = std::env::var_os("LM_E2E_CAPTURES") else {
        eprintln!("LM_E2E_CAPTURES is not set: no captures to replay");
        return;
    };
    let host = Host::new();
    let mut replayed = 0;
    for entry in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "lmcap" || ext == "lmraw") {
            let mut reader = minimap_runtime::services::CaptureReader::open(&path).unwrap();
            let world = minimap_store::structs::WorldIdentity::Local { world_dir: path.with_extension("world") };
            let mut replay = CaptureReplay::new(world);
            let runtime = std::sync::Arc::clone(host.session.as_ref().unwrap().runtime());
            assert!(replay.run(&runtime, &mut reader).unwrap() > 0, "{}", path.display());
            replayed += 1;
        }
    }
    eprintln!("{replayed} captures replayed");
    let _dir = host.shutdown();
}

#[test]
fn the_open_world_map_owns_the_pointer_over_the_map() {
    let mut host = Host::new();
    host.key(M, 0);
    for _ in 0..3 {
        host.frame();
    }

    host.pointer_move(900.0, 500.0);
    for _ in 0..3 {
        host.frame();
    }

    let owner = host.pointer_layer();
    assert!(owner.is_some_and(|(_, overlay)| overlay), "the map's layer owns the pointer: {owner:?}");
    let _dir = host.shutdown();
}

#[test]
fn a_tick_through_the_map_session_plans_a_scan_on_the_manager() {
    let mut host = Host::new();
    let mut session = MapSession::new(host.session.take().unwrap());
    let world = WorldIdentity::Local { world_dir: host.world_dir() };
    session.enter_world(&world, 0).unwrap();
    let input = TickInput {
        player: PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 },
        foreground: true,
        cursor_hidden: true,
        mouse_grabbed: true,
        ..TickInput::default()
    };
    let mut plan = ScanPlan::default();

    session.tick(&input, &mut plan);

    assert!(!plan.candidates.is_empty(), "the plan asks for chunks around the player");
    session.shutdown();
    let _dir = host.shutdown();
}
