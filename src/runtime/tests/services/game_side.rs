use std::time::{Duration, Instant};

use minimap_core::services::cave::LayerIds;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface};
use minimap_core::structs::input::{KeyPhase, KeyResult, KeybindAction, KeybindInput};
use minimap_core::structs::pos::RegionPos;
use minimap_runtime::services::{Runtime, WorldFlushes};
use minimap_runtime::structs::{SubmitStatus, TickInput};
use minimap_store::services::{BiomeKeyFile, CachePaths, FolderTurn};
use minimap_store::structs::WorldIdentity;

use crate::support::{Painted, SyntheticWorld};

fn bind(runtime: &Runtime, action: KeybindAction, mouse_grabbed: bool) -> KeyResult {
    runtime.keybind(KeybindInput { action, phase: KeyPhase::Pressed, mouse_grabbed })
}

fn draws_text(frame: &Painted, text: &str) -> bool {
    frame.mesh.draws_text(text)
}

#[test]
fn k_scans_the_band_around_the_feet_from_its_top_and_k_again_goes_back_to_the_surface() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 320);
    bind(&runtime, KeybindAction::ToggleCave, true);
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 95);
    assert!(world.plan.candidates.iter().all(|c| c.dim == LayerIds::cave(0, 5)));
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "Cave view on"));
    bind(&runtime, KeybindAction::ToggleCave, true);
    world.tick(&runtime);
    assert_eq!(world.plan.camera_height, 320);
    assert!(world.plan.candidates.iter().all(|c| c.dim == 0));
    runtime.shutdown();
}

#[test]
fn the_cave_badge_names_the_band_on_the_minimap_and_the_world_map() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(!draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    bind(&runtime, KeybindAction::ToggleCave, true);
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    bind(&runtime, KeybindAction::ToggleWorldMap, true);
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(world.frame.map_open());
    assert!(draws_text(&world.frame, "CAVE \u{b7} Y 80 to 95"));
    runtime.shutdown();
}

#[test]
fn a_cave_band_is_saved_in_its_own_layer_folder() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    bind(&runtime, KeybindAction::ToggleCave, true);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    runtime.leave_world();
    runtime.shutdown();
    let cache = world.world_dir().join("minimap_cache");
    let layer = LayerIds::cave(0, 5).to_string();
    assert!(cache.join("regions").join(&layer).exists() || cache.join("journal").join(&layer).exists());
    assert!(!cache.join("regions").join("0").exists());
}

#[test]
fn ctrl_k_hides_the_coordinates() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "8, 80, 8"));
    bind(&runtime, KeybindAction::ToggleCoordinates, true);
    world.draw(&runtime);
    assert!(!draws_text(&world.frame, "8, 80, 8"));
    assert!(draws_text(&world.frame, "Coordinates hidden"));
    runtime.shutdown();
}

// the store of a world held behind a turn the test owns, as if an earlier flush of the same folder were still running
fn held_world(world: &SyntheticWorld) -> (Runtime, FolderTurn) {
    let runtime = Runtime::new(world.config.clone(), world.paths(), world.grant).unwrap();
    let root = CachePaths::root(&world.identity(), &world.paths().data_dir);
    let held = runtime.folder_turns().take(&root);
    runtime.enter_world(&world.identity(), world.player.dim).unwrap();
    (runtime, held)
}

// one tick that probes nothing, so every chunk the store holds comes from the disk
fn tick_without_probes(world: &mut SyntheticWorld, runtime: &Runtime) {
    let input = TickInput { player: world.player, foreground: true, mouse_grabbed: true, ..TickInput::default() };
    runtime.tick(&input, &mut world.plan);
    for pos in world.plan.candidates.clone() {
        runtime.submit_chunk(pos, SubmitStatus::NotLoaded, &ChunkProbe::new(ChunkSurface::empty(), [0; 256]));
    }
}

#[test]
fn leaving_returns_while_the_world_flush_is_still_running() {
    let mut world = SyntheticWorld::new();
    let (runtime, held) = held_world(&world);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    let started = Instant::now();
    runtime.leave_world();
    let took = started.elapsed();
    assert!(took < Duration::from_secs(1), "the leave took {took:?}");
    assert!(!runtime.stats().in_world);
    drop(held);
    runtime.shutdown();
}

#[test]
fn a_rejoin_right_after_leaving_reads_every_chunk_written_before_the_leave() {
    let mut world = SyntheticWorld::new();
    let (runtime, held) = held_world(&world);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    let written = runtime.stats().surfaces_resident;
    assert!(written > 20, "{:?}", runtime.stats());
    runtime.leave_world();
    runtime.enter_world(&world.identity(), world.player.dim).unwrap();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(200));
        drop(held);
    });
    let deadline = Instant::now() + Duration::from_secs(10);
    while runtime.stats().surfaces_resident < written && Instant::now() < deadline {
        tick_without_probes(&mut world, &runtime);
        std::thread::sleep(Duration::from_millis(5));
    }
    release.join().unwrap();
    let stats = runtime.stats();
    assert!(stats.surfaces_resident >= written, "{written} written, {stats:?}");
    assert_eq!(stats.chunks_submitted, 0);
    runtime.shutdown();
}

#[test]
fn a_join_to_another_world_does_not_wait_for_the_old_flush() {
    let mut world = SyntheticWorld::new();
    let (runtime, held) = held_world(&world);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    let other = WorldIdentity::Local { world_dir: world.dir.path().join("other-world") };
    let started = Instant::now();
    runtime.leave_world();
    runtime.enter_world(&other, world.player.dim).unwrap();
    let took = started.elapsed();
    assert!(took < Duration::from_secs(1), "the switch took {took:?}");
    let written = world.run_until(&runtime, 2000, |rt, _| rt.stats().disk_written_bytes > 0);
    assert!(written, "the new world's store wrote nothing: {:?}", runtime.stats());
    drop(held);
    runtime.shutdown();
}

#[test]
fn shutdown_waits_for_the_world_flush_and_the_chunks_are_on_disk() {
    let mut world = SyntheticWorld::new();
    let (runtime, held) = held_world(&world);
    assert!(world.run_until(&runtime, 2000, |rt, _| rt.stats().bakes_completed > 4));
    runtime.leave_world();
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        drop(held);
    });
    let started = Instant::now();
    runtime.shutdown();
    let took = started.elapsed();
    release.join().unwrap();
    assert!(took >= Duration::from_millis(250), "shutdown returned after {took:?}, before the flush could run");
    let root = CachePaths::root(&world.identity(), &world.paths().data_dir);
    assert!(CachePaths::region_file(&root, RegionPos::new(0, 0, 0)).exists());
    assert!(CachePaths::region_file(&root, RegionPos::new(-1, -1, 0)).exists());
}

#[test]
fn a_rejoin_during_the_flush_keeps_the_biome_key_table_of_the_world_just_left() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    assert!(world.run_until(&runtime, 1000, |rt, _| rt.column_biome(8, 8).as_deref() == Some("plains")));
    // a turn after the running store's: the next store of the folder waits for it as for a flush still running
    let held = runtime.folder_turns().take(&CachePaths::root(&world.identity(), &world.paths().data_dir));
    runtime.leave_world();
    runtime.enter_world(&world.identity(), world.player.dim).unwrap();
    let named = world.run_until(&runtime, 500, |rt, _| rt.column_biome(8, 8).as_deref() == Some("plains"));
    assert!(named, "a chunk scanned while the old flush runs has no biome: {:?}", runtime.column_biome(8, 8));
    drop(held);
    runtime.shutdown();
    let keys = CachePaths::biome_keys_file(&CachePaths::root(&world.identity(), &world.paths().data_dir));
    let (_, names) = BiomeKeyFile::load(&keys).unwrap().expect("the key table is saved with the world");
    assert!(names.iter().any(|name| name == "plains"), "{names:?}");
}

#[test]
fn a_leave_past_the_flush_capacity_waits_only_a_bounded_time() {
    let world = SyntheticWorld::new();
    let runtime = Runtime::new(world.config.clone(), world.paths(), world.grant).unwrap();
    let mut held = Vec::new();
    let mut took = Duration::ZERO;
    for i in 0..=WorldFlushes::CAPACITY {
        let identity = WorldIdentity::Local { world_dir: world.dir.path().join(format!("world-{i}")) };
        held.push(runtime.folder_turns().take(&CachePaths::root(&identity, &world.paths().data_dir)));
        runtime.enter_world(&identity, 0).unwrap();
        let started = Instant::now();
        runtime.leave_world();
        took = started.elapsed();
    }
    assert!(took < Duration::from_millis(2500), "the leave past capacity took {took:?}");
    drop(held);
    runtime.shutdown();
}

#[test]
fn shutdown_also_waits_for_a_flush_left_alone_at_a_hand_off() {
    let mut world = SyntheticWorld::new();
    let runtime = Runtime::new(world.config.clone(), world.paths(), world.grant).unwrap();
    let mut roots = Vec::new();
    let mut held = Vec::new();
    for i in 0..=WorldFlushes::CAPACITY {
        let identity = WorldIdentity::Local { world_dir: world.dir.path().join(format!("world-{i}")) };
        let root = CachePaths::root(&identity, &world.paths().data_dir);
        held.push(runtime.folder_turns().take(&root));
        roots.push(root);
        runtime.enter_world(&identity, 0).unwrap();
        for _ in 0..5 {
            world.tick(&runtime);
        }
        runtime.leave_world();
    }
    // the oldest store stays held until shutdown has begun; the others go now
    let oldest = held.remove(0);
    drop(held);
    let release = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(300));
        drop(oldest);
    });
    runtime.shutdown();
    release.join().unwrap();
    for root in &roots {
        assert!(
            CachePaths::region_file(root, RegionPos::new(0, 0, 0)).exists(),
            "{} has no region file",
            root.display()
        );
    }
}
