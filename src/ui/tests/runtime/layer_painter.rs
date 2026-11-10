use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget};
use minimap_ui::runtime::LayerPainter;
use minimap_ui::testing::{HarnessEvent, PassOutput, UiHarness};

use crate::support::{Frames, Scene};

const ESCAPE: i32 = 0x1B;
const Z: i32 = 0x5A;

fn key(code: i32, ctrl: bool) -> [HarnessEvent; 2] {
    [
        HarnessEvent::Key { code, down: true, ctrl, shift: false },
        HarnessEvent::Key { code, down: false, ctrl, shift: false },
    ]
}

#[test]
fn escape_closes_the_world_map_when_no_menu_or_dialog_is_open() {
    let mut scene = Scene::new(false);
    scene.settle();

    scene.frame((1200.0, 700.0), false, &key(ESCAPE, false));

    assert_eq!(scene.actions, [MapAction::CloseWorldMap]);
}

#[test]
fn escape_closes_the_menu_and_not_the_map() {
    let mut scene = Scene::new(false);
    scene.ui.open_menu((300.0, 200.0), MenuTarget::Ground { dim: 0, x: 1, y: 64, z: 2 });
    scene.settle();

    scene.frame((1200.0, 700.0), false, &key(ESCAPE, false));

    assert_eq!(scene.actions, [MapAction::CloseMenu]);
}

#[test]
fn escape_while_typing_in_the_search_does_not_close_the_map() {
    let mut scene = Scene::new(true);
    scene.settle();
    scene.click("Search waypoints");

    scene.frame((1200.0, 700.0), false, &key(ESCAPE, false));

    assert!(!scene.actions.contains(&MapAction::CloseWorldMap), "{:?}", scene.actions);
}

#[test]
fn ctrl_z_undoes_a_removed_waypoint_while_the_undo_lasts() {
    let mut scene = Scene::new(false);
    scene.settle();
    scene.frame((1200.0, 700.0), false, &key(Z, true));
    let without_undo = scene.actions.clone();
    let id = scene.waypoints[0].id;
    let mut store = minimap_core::services::waypoints::WaypointStore::new(16, 1);
    for waypoint in &scene.waypoints {
        let _ = store.add(&waypoint.name, waypoint.dim, (waypoint.x, waypoint.y, waypoint.z), waypoint.source, 0);
    }
    scene.ui.open_menu((300.0, 200.0), MenuTarget::Marker(id));
    scene.ui.apply(MapAction::Choose(MenuItem::RemoveWaypoint), "", 0, &mut scene.view, &mut store, 0);
    assert!(scene.ui.can_undo(), "the removal offers an undo");

    scene.frame((1200.0, 700.0), false, &key(Z, true));

    assert!(without_undo.is_empty());
    assert_eq!(scene.actions, [MapAction::UndoRemove]);
}

#[test]
fn wheel_lines_count_the_notches_of_the_pass() {
    let mut harness = UiHarness::new();
    let mut lines = 0.0;
    let events = [HarnessEvent::Wheel(120), HarnessEvent::Wheel(120), HarnessEvent::Wheel(-120)];

    harness.run(&Frames::input((400.0, 300.0), false, &events), &mut PassOutput::new(), |ctx| {
        lines = LayerPainter::wheel_lines(ctx);
    });

    assert_eq!(lines, 1.0);
}

#[test]
fn a_panel_covers_the_point_under_it_and_the_map_does_not() {
    let mut scene = Scene::new(false);
    scene.settle();
    let tools = scene.centre_of("Chunk grid");

    let ctx = scene.harness.context().clone();

    assert!(LayerPainter::covers(&ctx, tools));
    assert!(!LayerPainter::covers(&ctx, (900.0, 500.0)));
}

// the manager routes a press to a layer only while its egui wants the pointer; the map is not a panel, but it
// still takes drags, right clicks and the wheel
#[test]
fn the_open_world_map_wants_the_pointer_over_the_map_too() {
    let mut scene = Scene::new(false);
    scene.settle();

    scene.frame((900.0, 500.0), false, &[]);
    scene.frame((900.0, 500.0), false, &[]);

    assert!(scene.mesh.wants_pointer);
}

#[test]
fn a_click_on_the_tools_still_reaches_them_after_a_press_on_the_map() {
    let mut scene = Scene::new(false);
    scene.settle();
    scene.frame((900.0, 500.0), true, &[]);
    scene.frame((900.0, 500.0), false, &[]);

    let actions = scene.click("Chunk grid");

    assert!(actions.contains(&MapAction::ToggleGrid), "{actions:?}");
}
