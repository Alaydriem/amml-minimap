use minimap_core::services::cave::LayerIds;
use minimap_core::structs::input::{KeyPhase, KeyResult, KeybindAction, KeybindInput};
use minimap_runtime::services::Runtime;

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
