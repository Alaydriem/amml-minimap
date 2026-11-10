use minimap_core::structs::ui::MapAction;
use minimap_ui::testing::HarnessEvent;

use crate::support::Scene;

#[test]
fn clicking_a_row_returns_centre_on_the_middle_of_its_block() {
    let mut scene = Scene::new(true);
    scene.settle();

    let actions = scene.click("Iron farm");

    assert_eq!(actions, vec![MapAction::CentreOn { x: 342.5, z: -30.5 }]);
}

#[test]
fn typing_in_the_search_filters_the_rows_and_holds_the_keyboard() {
    let mut scene = Scene::new(true);
    scene.settle();
    scene.click("Search waypoints");

    let typed = [HarnessEvent::Text('i'), HarnessEvent::Text('r')];
    scene.frame((1200.0, 700.0), false, &typed);
    scene.frame((1200.0, 700.0), false, &[]);

    assert_eq!(scene.painter.search(), "ir");
    assert!(scene.mesh.wants_keyboard);
    assert!(scene.mesh.draws_text("Iron farm"));
    assert!(!scene.mesh.draws_text("Village"));
}

#[test]
fn the_header_counts_the_dimension_and_rows_read_here_or_metres() {
    let mut scene = Scene::new(true);
    scene.settle();

    assert!(scene.mesh.draws_text("7 in this dimension"));
    assert!(scene.mesh.draws_text("Here"));
    assert!(scene.mesh.draws_text("140 m away"));
    assert!(scene.mesh.draws_text("342, -31"));
}

#[test]
fn the_close_button_returns_toggle_list() {
    let mut scene = Scene::new(true);
    scene.settle();

    let actions = scene.click("\u{e8bb}");

    assert_eq!(actions, vec![MapAction::ToggleList]);
}

#[test]
fn a_search_with_no_match_says_so() {
    let mut scene = Scene::new(true);
    scene.settle();
    scene.click("Search waypoints");

    scene.frame((1200.0, 700.0), false, &[HarnessEvent::Text('q')]);
    scene.frame((1200.0, 700.0), false, &[]);

    assert!(scene.mesh.draws_text("No waypoints here"));
}

#[test]
fn the_list_opens_under_the_tools_card_on_the_left() {
    let mut scene = Scene::new(true);
    scene.settle();

    let tools = scene.mesh.text_rect("Biome").unwrap_or_default();
    let search = scene.mesh.text_rect("Search waypoints").unwrap_or_default();

    assert!(search[1] > tools[3], "below the tools card: {search:?} after {tools:?}");
    assert!(search[0] < 100.0, "on the left: {search:?}");
}
