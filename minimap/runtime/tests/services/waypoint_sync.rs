use std::time::{Duration, Instant};

use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::draw::DrawCommand;
use minimap_core::structs::input::{KeyPhase, KeybindAction, KeybindInput};
use minimap_core::structs::waypoint::WaypointSource;
use minimap_runtime::services::{Runtime, WaypointSync};
use minimap_runtime::structs::TickInput;
use minimap_store::services::WaypointFile;

use crate::support::SyntheticWorld;

fn press_n(runtime: &Runtime) -> bool {
    let input = KeybindInput { action: KeybindAction::AddWaypoint, phase: KeyPhase::Pressed, mouse_grabbed: true };
    runtime.keybind(input).add_waypoint
}

fn tick_dead(world: &mut SyntheticWorld, runtime: &Runtime, dead: bool) {
    let input = TickInput {
        player: world.player,
        foreground: true,
        cursor_hidden: true,
        dead,
        mouse_grabbed: true,
        ..TickInput::default()
    };
    runtime.tick(&input, &mut world.plan);
}

// ticks until the waypoints file has been read, so changes are saved
fn wait_until_loaded(world: &mut SyntheticWorld, runtime: &Runtime) -> bool {
    world.run_until(runtime, 500, |rt, _| rt.waypoints_loaded())
}

#[test]
fn the_hotkey_adds_a_numbered_waypoint_at_the_player_block() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    assert!(press_n(&runtime));
    assert!(press_n(&runtime));
    let waypoints = runtime.waypoints();
    assert_eq!(waypoints.len(), 2);
    let first = &waypoints[0];
    assert_eq!((first.name.as_str(), first.dim, first.x, first.y, first.z), ("Waypoint 1", 0, 8, 80, 8));
    assert_eq!(first.source, WaypointSource::Hotkey);
    assert_eq!(waypoints[1].name, "Waypoint 2");
    runtime.shutdown();
}

#[test]
fn waypoints_survive_leaving_and_entering_the_world() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    assert!(wait_until_loaded(&mut world, &runtime));
    press_n(&runtime);
    runtime.leave_world();
    assert!(runtime.waypoints().is_empty());
    runtime.enter_world(&world.identity(), 0).unwrap();
    assert!(wait_until_loaded(&mut world, &runtime));
    assert_eq!(runtime.waypoints()[0].name, "Waypoint 1");
    runtime.shutdown();
    assert!(world.world_dir().join(WaypointFile::NAME).exists());
}

#[test]
fn a_death_sets_one_death_point_on_each_alive_to_dead_edge() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    tick_dead(&mut world, &runtime, false);
    tick_dead(&mut world, &runtime, true);
    world.player.x = 100.5;
    tick_dead(&mut world, &runtime, true);
    let deaths: Vec<_> = runtime.waypoints().into_iter().filter(|w| w.source == WaypointSource::Death).collect();
    assert_eq!(deaths.len(), 1);
    assert_eq!((deaths[0].x, deaths[0].name.as_str()), (8, WaypointStore::DEATH_NAME));
    tick_dead(&mut world, &runtime, false);
    tick_dead(&mut world, &runtime, true);
    let waypoints = runtime.waypoints();
    assert_eq!(waypoints.len(), 1);
    assert_eq!(waypoints[0].x, 100);
    runtime.shutdown();
}

#[test]
fn a_change_is_saved_after_the_save_delay_without_leaving() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    assert!(wait_until_loaded(&mut world, &runtime));
    press_n(&runtime);
    let file = world.world_dir().join(WaypointFile::NAME);
    let started = Instant::now();
    let saved = world.run_until(&runtime, 2000, |_, _| file.exists());
    assert!(saved, "nothing saved");
    assert!(started.elapsed() >= WaypointSync::SAVE_DELAY);
    assert!(started.elapsed() < WaypointSync::SAVE_DELAY + Duration::from_secs(2));
    runtime.shutdown();
}

#[test]
fn waypoints_are_saved_with_the_disk_cache_off() {
    let mut world = SyntheticWorld::new();
    world.config.terrain.enable_disk_cache = false;
    let runtime = world.runtime();
    assert!(wait_until_loaded(&mut world, &runtime));
    press_n(&runtime);
    runtime.leave_world();
    runtime.shutdown();
    assert!(world.world_dir().join(WaypointFile::NAME).exists());
    assert!(!world.world_dir().join("minimap_cache").exists());
}

#[test]
fn with_waypoints_turned_off_the_hotkey_passes_to_the_game_and_nothing_is_written() {
    let mut world = SyntheticWorld::new();
    world.config.waypoints.enabled = false;
    let runtime = world.runtime();
    world.tick(&runtime);
    assert!(!press_n(&runtime));
    tick_dead(&mut world, &runtime, true);
    assert!(runtime.waypoints().is_empty());
    runtime.leave_world();
    runtime.shutdown();
    assert!(!world.world_dir().join(WaypointFile::NAME).exists());
}

#[test]
fn a_hotkey_waypoint_is_drawn_on_the_minimap_with_its_name() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    press_n(&runtime);
    world.player.x += 20.0;
    let drawn = world.run_until(&runtime, 50, |_, frame| {
        frame.draw.commands.iter().any(|c| match c {
            DrawCommand::Text { start, len, .. } => frame.draw.text_of(*start, *len) == "Waypoint 1",
            _ => false,
        })
    });
    assert!(drawn);
    runtime.shutdown();
}
