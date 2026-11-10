use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget};
use minimap_ui::testing::HarnessEvent;

use crate::support::Scene;

fn open_dialog(scene: &mut Scene, target: MenuTarget) {
    let mut store = WaypointStore::new(16, 1);
    for w in &scene.waypoints {
        let _ = store.add(&w.name, w.dim, (w.x, w.y, w.z), w.source, 0);
    }
    let item = match target {
        MenuTarget::Ground { .. } => MenuItem::AddWaypoint,
        MenuTarget::Marker(_) => MenuItem::EditWaypoint,
    };
    scene.ui.open_menu((900.0, 400.0), target);
    scene.ui.apply(MapAction::Choose(item), "", 0, &mut scene.view, &mut store, 0);
    scene.settle();
}

const GROUND: MenuTarget = MenuTarget::Ground { dim: 0, x: 316, y: 71, z: -58 };

fn key(code: i32) -> HarnessEvent {
    HarnessEvent::Key { code, down: true, ctrl: false, shift: false }
}

#[test]
fn a_name_typed_then_enter_in_one_frame_saves_the_whole_name() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);

    let events =
        [HarnessEvent::Text('B'), HarnessEvent::Text('a'), HarnessEvent::Text('s'), HarnessEvent::Text('e'), key(0x0D)];
    scene.frame((1200.0, 700.0), false, &events);

    assert!(scene.actions.contains(&MapAction::SaveDialog), "{:?}", scene.actions);
    assert_eq!(scene.painter.drafts().name, "Base");
    assert!(!scene.mesh.wants_keyboard, "the keyboard goes back to the game");
}

#[test]
fn escape_cancels_the_dialog() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);

    scene.frame((1200.0, 700.0), false, &[key(0x1B)]);

    assert_eq!(scene.actions, vec![MapAction::CancelDialog]);
}

#[test]
fn the_dialog_shows_its_heading_position_and_starts_from_the_waypoint_name() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);
    let new = scene.mesh.draws_text("New waypoint") && scene.mesh.draws_text("X 316   Y 71   Z -58");

    let mut store = WaypointStore::new(8, 1);
    scene.ui.apply(MapAction::CancelDialog, "", 0, &mut scene.view, &mut store, 0);
    let id = scene.waypoints[2].id;
    open_dialog(&mut scene, MenuTarget::Marker(id));

    assert!(new);
    assert!(scene.mesh.draws_text("Edit waypoint"));
    assert_eq!(scene.painter.drafts().name, "Village");
}

#[test]
fn the_switch_labels_return_their_actions() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);

    let edge = scene.click("Arrow at the edge when far away");
    let distance = scene.click("Show distance on the arrow");
    let save = scene.click("Save");

    assert_eq!((edge, distance), (vec![MapAction::DialogEdge], vec![MapAction::DialogDistance]));
    assert!(save.contains(&MapAction::SaveDialog));
}

#[test]
fn the_dimmed_map_takes_presses_off_the_map() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);

    scene.frame((40.0, 700.0), false, &[]);
    scene.frame((40.0, 700.0), true, &[]);

    assert!(scene.mesh.wants_pointer);
    assert!(scene.actions.is_empty(), "{:?}", scene.actions);
}

#[test]
fn a_swatch_click_returns_its_colour() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);
    let label = scene.mesh.text_rect("COLOUR").unwrap_or_default();
    // the first swatch sits under the label's left edge, 18 points in
    let at = (label[0] + 18.0, label[3] + 26.0);

    scene.frame(at, false, &[]);
    scene.frame(at, true, &[]);
    scene.frame(at, false, &[]);

    assert_eq!(scene.actions, vec![MapAction::DialogColour(WaypointStore::PALETTE[0])]);
}

#[test]
fn the_second_row_of_swatches_holds_the_rest_of_the_palette() {
    let mut scene = Scene::new(false);
    open_dialog(&mut scene, GROUND);
    let label = scene.mesh.text_rect("COLOUR").unwrap_or_default();
    // the first swatch of the second row: one row (40 points) below the first
    let at = (label[0] + 18.0, label[3] + 26.0 + 40.0);

    scene.frame(at, false, &[]);
    scene.frame(at, true, &[]);
    scene.frame(at, false, &[]);

    assert_eq!(scene.actions, vec![MapAction::DialogColour(WaypointStore::PALETTE[6])]);
}
