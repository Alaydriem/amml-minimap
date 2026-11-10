use minimap_core::config::Config;
use minimap_core::services::input::InputController;
use minimap_core::structs::input::{KeyPhase, KeyResult, KeybindAction, KeybindInput};
use minimap_core::structs::view::ViewState;

fn controller() -> InputController {
    InputController::new(&Config::default())
}

fn press(action: KeybindAction) -> KeybindInput {
    KeybindInput { action, phase: KeyPhase::Pressed, mouse_grabbed: true }
}

fn in_chat(action: KeybindAction) -> KeybindInput {
    KeybindInput { mouse_grabbed: false, ..press(action) }
}

fn open_map() -> ViewState {
    ViewState { world_map_open: true, ..ViewState::new(5) }
}

#[test]
fn toggle_world_map_opens_the_map_and_resets_the_view() {
    let mut view = ViewState { offset_x: 40.0, anchor: Some((5.0, 5.0)), ..ViewState::new(5) };

    controller().on_keybind(&mut view, press(KeybindAction::ToggleWorldMap));

    assert!(view.world_map_open);
    assert_eq!((view.offset_x, view.anchor), (0.0, None));
}

#[test]
fn toggle_world_map_closes_the_open_map_and_clears_the_anchor() {
    let mut view = ViewState { anchor: Some((7.0, 7.0)), ..open_map() };

    controller().on_keybind(&mut view, in_chat(KeybindAction::ToggleWorldMap));

    assert!(!view.world_map_open);
    assert_eq!(view.anchor, None);
}

#[test]
fn toggle_minimap_works_on_either_view() {
    let mut view = ViewState::new(5);
    controller().on_keybind(&mut view, press(KeybindAction::ToggleMinimap));
    assert!(!view.show_minimap);

    let mut map = open_map();
    controller().on_keybind(&mut map, in_chat(KeybindAction::ToggleMinimap));
    assert!(!map.show_minimap && map.world_map_open);
}

#[test]
fn zoom_changes_the_radius_within_its_limits() {
    let mut view = ViewState::new(3);
    let zoom_in = controller().on_keybind(&mut view, press(KeybindAction::ZoomIn));
    assert!(zoom_in.radius_changed);
    assert_eq!(view.minimap_radius, 2);

    let at_limit = controller().on_keybind(&mut view, press(KeybindAction::ZoomIn));
    assert!(!at_limit.radius_changed);
    assert_eq!(view.minimap_radius, 2);

    view.minimap_radius = 16;
    controller().on_keybind(&mut view, press(KeybindAction::ZoomOut));
    assert_eq!(view.minimap_radius, 16);
}

#[test]
fn zoom_repeats_while_held() {
    let mut view = ViewState::new(8);
    let repeat = KeybindInput { phase: KeyPhase::Repeat, ..press(KeybindAction::ZoomOut) };

    controller().on_keybind(&mut view, repeat);
    controller().on_keybind(&mut view, repeat);

    assert_eq!(view.minimap_radius, 10);
}

#[test]
fn zoom_and_add_waypoint_do_nothing_on_the_open_map() {
    let mut view = open_map();

    let zoom = controller().on_keybind(&mut view, press(KeybindAction::ZoomIn));
    let add = controller().on_keybind(&mut view, press(KeybindAction::AddWaypoint));

    assert_eq!((zoom, add), (KeyResult::default(), KeyResult::default()));
    assert_eq!(view.minimap_radius, 5);
}

#[test]
fn add_waypoint_asks_for_a_waypoint_on_the_game_view() {
    let mut view = ViewState::new(5);

    assert!(controller().on_keybind(&mut view, press(KeybindAction::AddWaypoint)).add_waypoint);
}

#[test]
fn chat_open_ignores_add_waypoint_and_cave() {
    let mut view = ViewState::new(5);

    let add = controller().on_keybind(&mut view, in_chat(KeybindAction::AddWaypoint));
    let cave = controller().on_keybind(&mut view, in_chat(KeybindAction::ToggleCave));
    let zoom = controller().on_keybind(&mut view, in_chat(KeybindAction::ZoomIn));
    let open = controller().on_keybind(&mut view, in_chat(KeybindAction::ToggleWorldMap));

    assert_eq!([add, cave, zoom, open], [KeyResult::default(); 4]);
    assert_eq!((view.minimap_radius, view.world_map_open), (5, false));
}

#[test]
fn add_waypoint_is_ignored_when_waypoints_are_disabled() {
    let mut config = Config::default();
    config.waypoints.enabled = false;
    let mut view = ViewState::new(5);

    let result = InputController::new(&config).on_keybind(&mut view, press(KeybindAction::AddWaypoint));

    assert!(!result.add_waypoint);
}

#[test]
fn toggle_cave_works_on_either_view() {
    let mut view = ViewState::new(5);
    assert!(controller().on_keybind(&mut view, press(KeybindAction::ToggleCave)).toggle_cave);

    let mut map = open_map();
    assert!(controller().on_keybind(&mut map, in_chat(KeybindAction::ToggleCave)).toggle_cave);
}

#[test]
fn toggle_coordinates_works_on_either_view() {
    let mut view = ViewState::new(5);
    let hide = controller().on_keybind(&mut view, press(KeybindAction::ToggleCoordinates));
    assert!(hide.coordinates_changed && !view.show_coordinates);

    let mut map = ViewState { show_coordinates: false, ..open_map() };
    let show = controller().on_keybind(&mut map, in_chat(KeybindAction::ToggleCoordinates));
    assert!(show.coordinates_changed && map.show_coordinates);
}

#[test]
fn released_never_acts() {
    for action in KeybindAction::ALL {
        let mut view = ViewState::new(5);
        let released = KeybindInput { phase: KeyPhase::Released, ..press(action) };

        let result = controller().on_keybind(&mut view, released);

        assert_eq!(result, KeyResult::default(), "{action:?}");
        assert_eq!(view, ViewState::new(5), "{action:?}");
    }
}

#[test]
fn repeat_acts_only_for_zoom() {
    for action in [KeybindAction::ToggleWorldMap, KeybindAction::ToggleMinimap, KeybindAction::ToggleCave] {
        let mut view = ViewState::new(5);
        let repeat = KeybindInput { phase: KeyPhase::Repeat, ..press(action) };

        controller().on_keybind(&mut view, repeat);

        assert_eq!(view, ViewState::new(5), "{action:?}");
    }
}

#[test]
fn closing_the_world_map_clears_the_anchor() {
    let mut view = ViewState { anchor: Some((7.0, 7.0)), ..open_map() };

    InputController::close_world_map(&mut view);

    assert!(!view.world_map_open);
    assert_eq!(view.anchor, None);
}

#[test]
fn wheel_zooms_by_zoom_step_per_line_and_clamps() {
    let mut view = open_map();
    controller().on_wheel(&mut view, 1.0);
    assert!((view.zoom - 1.15).abs() < 1e-6);

    controller().on_wheel(&mut view, -2.0);
    assert!((view.zoom - 1.0 / 1.15).abs() < 1e-5, "{}", view.zoom);

    controller().on_wheel(&mut view, -200.0);
    assert!((view.zoom - 0.2).abs() < 1e-6);
}

#[test]
fn wheel_does_nothing_on_a_closed_map() {
    let mut view = ViewState::new(5);

    controller().on_wheel(&mut view, 3.0);

    assert_eq!(view.zoom, ViewState::new(5).zoom);
}

#[test]
fn dragging_moves_the_map_against_the_cursor_scaled_by_zoom() {
    let mut view = ViewState { world_map_open: true, zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, -4.0, 0.0);
    assert_eq!((view.offset_x, view.offset_z), (-5.0, 2.0));
}

#[test]
fn on_a_turned_map_the_drag_still_moves_the_map_under_the_cursor() {
    // a quarter turn puts world east at the screen bottom, so screen right is world north;
    // dragging right pulls the map right, so the centre moves south
    let mut view = ViewState { world_map_open: true, zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, 0.0, std::f32::consts::FRAC_PI_2);
    assert!(view.offset_x.abs() < 1e-4, "{}", view.offset_x);
    assert!((view.offset_z - 5.0).abs() < 1e-4, "{}", view.offset_z);
}

#[test]
fn a_drag_moves_the_anchor_and_not_the_offset() {
    let mut view = ViewState { world_map_open: true, anchor: Some((0.0, 0.0)), zoom: 2.0, ..ViewState::new(5) };
    controller().on_drag(&mut view, 10.0, -4.0, 0.0);
    assert_eq!(view.anchor, Some((-5.0, 2.0)));
    assert_eq!((view.offset_x, view.offset_z), (0.0, 0.0));
}
