use amml_ui::icon::Icon;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::ui::{MapAction, MenuItem, MenuTarget};

use crate::support::{IconText, Scene};

const GROUND: MenuTarget = MenuTarget::Ground { dim: 0, x: 316, y: 71, z: -58 };

fn open_menu(scene: &mut Scene, target: MenuTarget) {
    scene.ui.open_menu((900.0, 400.0), target);
    scene.settle();
}

#[test]
fn the_header_names_the_block_and_a_marker_menu_names_the_waypoint() {
    let mut scene = Scene::new(false);
    open_menu(&mut scene, GROUND);
    let ground = scene.mesh.draws_text("BLOCK 316, -58");

    let id = scene.waypoints[1].id;
    open_menu(&mut scene, MenuTarget::Marker(id));

    assert!(ground);
    assert!(scene.mesh.draws_text("IRON FARM"));
    assert!(scene.mesh.draws_text("Edit") && scene.mesh.draws_text("Remove"));
}

#[test]
fn a_row_click_returns_choose_for_its_item() {
    let mut scene = Scene::new(false);
    open_menu(&mut scene, GROUND);

    let actions = scene.click("Copy coordinates");

    assert_eq!(actions, vec![MapAction::Choose(MenuItem::CopyCoordinates)]);
}

#[test]
fn the_waypoint_switch_names_what_it_will_do() {
    let mut scene = Scene::new(false);
    open_menu(&mut scene, GROUND);
    let shown = scene.mesh.draws_text("Hide waypoints") && scene.mesh.draws_text(&IconText::of(Icon::Hide));

    let mut store = WaypointStore::new(8, 1);
    scene.ui.apply(MapAction::Choose(MenuItem::ToggleWaypoints), "", 0, &mut scene.view, &mut store, 0);
    open_menu(&mut scene, GROUND);

    assert!(shown);
    assert!(scene.mesh.draws_text("Show waypoints") && scene.mesh.draws_text(&IconText::of(Icon::Show)));
}

#[test]
fn a_left_press_outside_the_menu_closes_it() {
    let mut scene = Scene::new(false);
    open_menu(&mut scene, GROUND);

    scene.frame((1200.0, 650.0), false, &[]);
    scene.frame((1200.0, 650.0), true, &[]);

    assert!(scene.actions.contains(&MapAction::CloseMenu));
}
