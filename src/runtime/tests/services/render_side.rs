use minimap_core::structs::draw::{DrawCommand, TextureGrant, TextureOp};
use minimap_core::structs::input::{KeyPhase, KeyResult, KeybindAction, KeybindInput};
use minimap_core::structs::settings::{SettingValue, SettingWrite};
use minimap_runtime::services::{ExportService, Runtime};
use minimap_ui::testing::HarnessEvent;

use crate::support::{Painted, SyntheticWorld};

const CENTRE: (f32, f32) = SyntheticWorld::CENTRE;

fn frame(world: &mut SyntheticWorld, runtime: &Runtime, cursor: (f32, f32), left: bool, right: bool) {
    world.paint(runtime, cursor, left, right, &[]);
}

// a frame with `events` for the world map's layer, then one with none
fn events(world: &mut SyntheticWorld, runtime: &Runtime, events: &[HarnessEvent]) {
    world.paint(runtime, CENTRE, false, false, events);
    world.paint(runtime, CENTRE, false, false, &[]);
}

// a key pressed and released on the world map's layer
fn key(world: &mut SyntheticWorld, runtime: &Runtime, code: i32, ctrl: bool) {
    let down = HarnessEvent::Key { code, down: true, ctrl, shift: false };
    let up = HarnessEvent::Key { code, down: false, ctrl, shift: false };
    events(world, runtime, &[down, up]);
}

fn type_text(world: &mut SyntheticWorld, runtime: &Runtime, text: &str) {
    let typed: Vec<HarnessEvent> = text.chars().map(HarnessEvent::Text).collect();
    events(world, runtime, &typed);
}

fn region_uploads(frame: &Painted) -> usize {
    frame.textures.iter().filter(|op| matches!(op, TextureOp::Upload { tex, .. } if *tex > 1)).count()
}

fn click(world: &mut SyntheticWorld, runtime: &Runtime, cursor: (f32, f32), right: bool) {
    frame(world, runtime, cursor, !right, right);
    frame(world, runtime, cursor, false, false);
}

// a right click at the screen centre, then a left click on the menu row named `item`
fn menu_pick(world: &mut SyntheticWorld, runtime: &Runtime, item: &str) {
    open_menu(world, runtime, CENTRE);
    click_text(world, runtime, item);
}

// a right click at `at`, then frames until the menu is laid out
fn open_menu(world: &mut SyntheticWorld, runtime: &Runtime, at: (f32, f32)) {
    click(world, runtime, at, true);
    frame(world, runtime, at, false, false);
    frame(world, runtime, at, false, false);
}

fn bind(runtime: &Runtime, action: KeybindAction, mouse_grabbed: bool) -> KeyResult {
    runtime.keybind(KeybindInput { action, phase: KeyPhase::Pressed, mouse_grabbed })
}

fn draws_text(frame: &Painted, text: &str) -> bool {
    frame.mesh.draws_text(text)
}

// the hover value of a block, as the info rows show it
fn block(x: i32, z: i32) -> String {
    format!("X {x}  Z {z}")
}

// the centre of the last text that is exactly `text` in the last frame; panels are drawn after the
// map, so a list row wins over a marker label with the same name
fn text_at(world: &SyntheticWorld, text: &str) -> (f32, f32) {
    let mesh = &world.frame.mesh;
    let found = mesh.text_rects.iter().rev().find(|t| mesh.text_of(t) == text);
    let rect = found.map(|t| t.rect).unwrap_or_else(|| panic!("{text} in {:?}", texts(&world.frame)));
    ((rect[0] + rect[2]) * 0.5, (rect[1] + rect[3]) * 0.5)
}

// moves the cursor onto a text, so the UI sees it hovered, clicks it, then draws two frames with
// the cursor on the map: a panel that opens is laid out unseen in its first frame
fn click_text(world: &mut SyntheticWorld, runtime: &Runtime, text: &str) {
    let at = text_at(world, text);
    frame(world, runtime, at, false, false);
    click(world, runtime, at, false);
    frame(world, runtime, CENTRE, false, false);
    frame(world, runtime, CENTRE, false, false);
}

fn open_world_map(world: &mut SyntheticWorld, runtime: &Runtime) {
    bind(runtime, KeybindAction::ToggleWorldMap, true);
    for _ in 0..30 {
        frame(world, runtime, CENTRE, false, false);
    }
}

// a world with explored chunks around the player and the world map open; exports go to the test folder
fn explored_map(world: &mut SyntheticWorld) -> Runtime {
    let exports = world.paths().data_dir.join(ExportService::FOLDER);
    world.config.export.folder = exports.to_string_lossy().into_owned();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    open_world_map(world, &runtime);
    runtime
}

// right-clicks the screen centre, picks Copy coordinates, and returns the copied text
fn copied_at_centre(world: &mut SyntheticWorld, runtime: &Runtime) -> String {
    open_menu(world, runtime, CENTRE);
    let at = text_at(world, "Copy coordinates");
    frame(world, runtime, at, false, false);
    frame(world, runtime, at, true, false);
    frame(world, runtime, at, false, false);
    let copied = runtime.take_clipboard().unwrap_or_default();
    frame(world, runtime, CENTRE, false, false);
    copied
}

#[test]
fn add_here_types_a_name_and_saves_a_waypoint_on_the_clicked_column_at_its_height() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    menu_pick(&mut world, &runtime, "Add waypoint here");
    assert!(draws_text(&world.frame, "New waypoint"));
    type_text(&mut world, &runtime, "Base");
    assert!(world.frame.map_open());
    key(&mut world, &runtime, ENTER, false);
    frame(&mut world, &runtime, CENTRE, false, false);
    let saved = runtime.waypoints();
    assert_eq!(saved.len(), 1, "{:?}", texts(&world.frame));
    assert_eq!(saved[0].name, "Base");
    assert!((saved[0].x - 8).abs() <= 1 && (saved[0].z - 8).abs() <= 1, "{:?}", saved[0]);
    assert_eq!(saved[0].y, i32::from(SyntheticWorld::height(saved[0].x, saved[0].z)));
    runtime.shutdown();
}

#[test]
fn copy_coordinates_puts_its_text_on_the_clipboard_once() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    let copied = copied_at_centre(&mut world, &runtime);
    assert!(copied.contains(", ~, "), "{copied}");
    assert!(draws_text(&world.frame, &format!("Copied {copied}")));
    assert_eq!(runtime.take_clipboard(), None);
    runtime.shutdown();
}

#[test]
fn a_press_that_closes_the_menu_does_not_pan_the_map() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    let before = copied_at_centre(&mut world, &runtime);
    click(&mut world, &runtime, CENTRE, true);
    frame(&mut world, &runtime, (100.0, 100.0), true, false);
    for step in 1..=10 {
        frame(&mut world, &runtime, (100.0 + step as f32 * 20.0, 100.0), true, false);
    }
    frame(&mut world, &runtime, (300.0, 100.0), false, false);
    assert_eq!(copied_at_centre(&mut world, &runtime), before);
    runtime.shutdown();
}

#[test]
fn a_marker_under_the_cursor_can_be_removed_and_ctrl_z_brings_it_back() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 50, |rt, _| rt.waypoints_loaded());
    bind(&runtime, KeybindAction::AddWaypoint, true);
    open_world_map(&mut world, &runtime);
    menu_pick(&mut world, &runtime, "Remove");
    assert!(runtime.waypoints().is_empty());
    assert!(draws_text(&world.frame, "Removed \"Waypoint 1\" - press Ctrl+Z to undo"));
    key(&mut world, &runtime, Z, true);
    assert_eq!(runtime.waypoints().len(), 1);
    runtime.shutdown();
}

#[test]
fn hiding_waypoints_takes_them_off_the_minimap_too() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 50, |rt, _| rt.waypoints_loaded());
    bind(&runtime, KeybindAction::AddWaypoint, true);
    world.player.x += 20.0;
    assert!(world.run_until(&runtime, 50, |_, frame| draws_text(frame, "Waypoint 1")));
    open_world_map(&mut world, &runtime);
    menu_pick(&mut world, &runtime, "Hide waypoints");
    bind(&runtime, KeybindAction::ToggleWorldMap, false);
    for _ in 0..5 {
        world.tick(&runtime);
        world.draw(&runtime);
        assert!(!draws_text(&world.frame, "Waypoint 1"));
    }
    assert!(draws_text(&world.frame, "Waypoints hidden"));
    runtime.shutdown();
}

#[test]
fn a_refused_hotkey_says_why_on_the_minimap() {
    let mut world = SyntheticWorld::new();
    world.config.waypoints.max_per_world = 1;
    let runtime = world.runtime();
    world.tick(&runtime);
    bind(&runtime, KeybindAction::AddWaypoint, true);
    bind(&runtime, KeybindAction::AddWaypoint, true);
    world.draw(&runtime);
    assert!(draws_text(&world.frame, "This world already has 1 waypoints"));
    assert_eq!(runtime.waypoints().len(), 1);
    runtime.shutdown();
}

// ticks and draws until the message line starts with `prefix`; returns the whole message
fn wait_for_message(world: &mut SyntheticWorld, runtime: &Runtime, prefix: &str) -> Option<String> {
    for _ in 0..1500 {
        world.tick(runtime);
        frame(world, runtime, (5.0, 5.0), false, false);
        let found = texts(&world.frame).into_iter().find(|text| text.starts_with(prefix));
        if found.is_some() {
            return found;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    None
}

fn exported_file(world: &SyntheticWorld, message: &str) -> std::path::PathBuf {
    let name = message.strip_prefix("Exported ").expect("an export message");
    world.paths().data_dir.join(ExportService::FOLDER).join(name)
}

fn png_size(path: &std::path::Path) -> (u32, u32) {
    let reader = png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap())).read_info().unwrap();
    (reader.info().width, reader.info().height)
}

#[test]
fn export_explored_area_from_the_menu_writes_every_explored_region() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    world.run_until(&runtime, 500, |rt, _| rt.stats().bakes_completed > 8);
    menu_pick(&mut world, &runtime, "Export explored area");
    let message = wait_for_message(&mut world, &runtime, "Exported ").expect("the export finished");
    let file = exported_file(&world, &message);
    assert!(message.ends_with("-area.png"), "{message}");
    let (width, height) = png_size(&file);
    assert!(width >= 256 && width % 256 == 0 && height % 256 == 0, "{width} x {height}");
    runtime.shutdown();
}

// the RGBA of one pixel of a PNG
fn png_pixel(path: &std::path::Path, x: u32, y: u32) -> [u8; 4] {
    let mut reader =
        png::Decoder::new(std::io::BufReader::new(std::fs::File::open(path).unwrap())).read_info().unwrap();
    let mut pixels = vec![0; reader.output_buffer_size().unwrap()];
    let info = reader.next_frame(&mut pixels).unwrap();
    let at = ((y * info.width + x) * 4) as usize;
    [pixels[at], pixels[at + 1], pixels[at + 2], pixels[at + 3]]
}

#[test]
fn export_this_view_writes_the_regions_on_screen() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    world.run_until(&runtime, 500, |rt, _| rt.stats().bakes_completed > 8);

    menu_pick(&mut world, &runtime, "Export this view");
    let message = wait_for_message(&mut world, &runtime, "Exported ").expect("the export finished");

    assert!(message.ends_with("-view.png"), "{message}");
    let (width, height) = png_size(&exported_file(&world, &message));
    assert!(width >= 256 && width % 256 == 0 && height % 256 == 0, "{width} x {height}");
    runtime.shutdown();
}

#[test]
fn a_view_export_leaves_unexplored_regions_see_through() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    world.run_until(&runtime, 500, |rt, _| rt.stats().bakes_completed > 8);

    menu_pick(&mut world, &runtime, "Export this view");
    let message = wait_for_message(&mut world, &runtime, "Exported ").expect("the export finished");

    let file = exported_file(&world, &message);
    assert_eq!(png_pixel(&file, 0, 0)[3], 0, "the view's corner is beyond the explored land");
    runtime.shutdown();
}

#[test]
fn a_view_export_respects_max_pixels() {
    let mut world = SyntheticWorld::new();
    world.config.export.max_pixels = 256;
    let runtime = explored_map(&mut world);
    world.run_until(&runtime, 500, |rt, _| rt.stats().bakes_completed > 8);

    menu_pick(&mut world, &runtime, "Export this view");
    let message = wait_for_message(&mut world, &runtime, "Exported ").expect("the export finished");

    let (width, height) = png_size(&exported_file(&world, &message));
    assert!(width <= 256 && height <= 256, "{width} x {height}");
    runtime.shutdown();
}

#[test]
fn text_grows_with_a_taller_screen() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    world.display = (3840.0, 2160.0);
    world.draw(&runtime);
    let sizes: Vec<f32> = world
        .frame
        .draw
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { size, .. } => Some(*size),
            _ => None,
        })
        .collect();
    assert!(!sizes.is_empty() && sizes.iter().all(|s| (*s - 36.0).abs() < 1e-3), "{sizes:?}");
    runtime.shutdown();
}

const ESCAPE: i32 = 0x1B;
const ENTER: i32 = 0x0D;
const Z: i32 = 0x5A;
fn texts(frame: &Painted) -> Vec<String> {
    frame.mesh.text_rects.iter().map(|t| frame.mesh.text_of(t).to_string()).collect()
}

fn lines(frame: &Painted) -> usize {
    frame.draw.commands.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count()
}

// one frame with the cursor at `at`; the texts it drew
fn hover(world: &mut SyntheticWorld, runtime: &Runtime, at: (f32, f32)) -> Vec<String> {
    frame(world, runtime, at, false, false);
    texts(&world.frame)
}

// "Hide waypoints" from the ground menu, opened away from the markers at the player
fn hide_waypoints(world: &mut SyntheticWorld, runtime: &Runtime) {
    open_menu(world, runtime, (900.0, 500.0));
    click_text(world, runtime, "Hide waypoints");
    assert!(draws_text(&world.frame, "Waypoints hidden"), "{:?}", texts(&world.frame));
}

// hotkey waypoints at the player, with the map closed
fn add_hotkey_waypoints(world: &mut SyntheticWorld, runtime: &Runtime, count: usize) {
    for _ in 0..count {
        bind(runtime, KeybindAction::AddWaypoint, true);
        world.tick(runtime);
    }
}

#[test]
fn the_info_block_names_the_block_chunk_and_biome_under_the_cursor() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    let named = world.run_until(&runtime, 300, |_, f| draws_text(f, "Plains"));
    assert!(named, "{:?}", texts(&world.frame));
    let lines = hover(&mut world, &runtime, CENTRE);
    for text in [block(8, 8).as_str(), "0, 0", "Plains", "Chunk grid", "Slime chunks", "Waypoints", "Overworld"] {
        assert!(lines.iter().any(|t| t == text), "{text} in {lines:?}");
    }
    runtime.shutdown();
}

#[test]
fn the_hover_lines_hide_over_the_tools_block() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    let at = text_at(&world, "Chunk grid");
    hover(&mut world, &runtime, at);
    let lines = hover(&mut world, &runtime, at);
    assert!(!lines.iter().any(|t| t.starts_with("X ")), "{lines:?}");
    runtime.shutdown();
}

#[test]
fn hover_over_land_known_only_from_disk_shows_its_biome_after_a_restart() {
    let mut world = SyntheticWorld::new();
    {
        let runtime = world.runtime();
        assert!(world.run_until(&runtime, 300, |rt, _| rt.column_biome(8, 8).as_deref() == Some("plains")
            && rt.waypoints_loaded()));
        add_hotkey_waypoints(&mut world, &runtime, 1);
        runtime.leave_world();
        runtime.shutdown();
    }
    world.player.x = 5000.5;
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.waypoints_loaded());
    open_world_map(&mut world, &runtime);
    click_text(&mut world, &runtime, "Waypoints");
    click_text(&mut world, &runtime, "Waypoint 1");
    let mut found = false;
    for _ in 0..300 {
        world.tick(&runtime);
        let lines = hover(&mut world, &runtime, CENTRE);
        if lines.iter().any(|t| t == "Plains") {
            assert!(lines.iter().any(|t| *t == block(8, 8)), "{lines:?}");
            found = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    assert!(found, "{:?}", texts(&world.frame));
    runtime.shutdown();
}

#[test]
fn a_click_on_the_tools_block_toggles_the_grid_and_does_not_pan() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    // the grid covers explored chunks only, which the baked region tiles show
    assert!(world.run_until(&runtime, 500, |rt, _| rt.stats().tiles_resident > 0));
    for _ in 0..5 {
        frame(&mut world, &runtime, CENTRE, false, false);
    }
    frame(&mut world, &runtime, CENTRE, false, false);
    let before = lines(&world.frame);
    let row = text_at(&world, "Chunk grid");
    frame(&mut world, &runtime, row, false, false);
    click(&mut world, &runtime, row, false);
    frame(&mut world, &runtime, CENTRE, false, false);
    assert!(lines(&world.frame) > before, "the grid is drawn over the explored chunks");
    frame(&mut world, &runtime, row, true, false);
    for i in 1..=10 {
        let t = i as f32 / 10.0;
        let at = (row.0 + (600.0 - row.0) * t, row.1 + (400.0 - row.1) * t);
        frame(&mut world, &runtime, at, true, false);
    }
    frame(&mut world, &runtime, CENTRE, false, false);
    let lines = hover(&mut world, &runtime, CENTRE);
    assert!(lines.iter().any(|t| *t == block(8, 8)), "a press on the panel does not pan: {lines:?}");
    runtime.shutdown();
}

#[test]
fn a_right_click_on_the_tools_block_or_list_opens_no_menu() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    click_text(&mut world, &runtime, "Waypoints");
    for text in ["Chunk grid", "Search waypoints"] {
        let at = text_at(&world, text);
        frame(&mut world, &runtime, at, false, false);
        click(&mut world, &runtime, at, true);
        assert!(!draws_text(&world.frame, "Copy coordinates"), "a right click on {text} opens no menu");
    }
    runtime.shutdown();
}

#[test]
fn a_wheel_turn_over_the_list_scrolls_it_and_does_not_zoom() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    add_hotkey_waypoints(&mut world, &runtime, 15);
    open_world_map(&mut world, &runtime);
    hide_waypoints(&mut world, &runtime);
    click_text(&mut world, &runtime, "Waypoints");
    let row = text_at(&world, "Waypoint 2");
    frame(&mut world, &runtime, row, false, false);
    frame(&mut world, &runtime, row, false, false);
    assert!(draws_text(&world.frame, "Waypoint 1"));
    let wheel = [HarnessEvent::Wheel(-120), HarnessEvent::Wheel(-120), HarnessEvent::Wheel(-120)];
    world.paint(&runtime, row, false, false, &wheel);
    for _ in 0..30 {
        frame(&mut world, &runtime, row, false, false);
    }
    let lines = texts(&world.frame);
    assert!(!lines.iter().any(|t| t == "Waypoint 1"), "{lines:?}");
    assert!(lines.iter().any(|t| t.contains("Zoom: 1.00x")), "{lines:?}");
    runtime.shutdown();
}

// a waypoint at (40, 64, 8): the list picks it
fn pick_far_waypoint(world: &mut SyntheticWorld, runtime: &Runtime) {
    world.player.x = 40.5;
    world.tick(runtime);
    add_hotkey_waypoints(world, runtime, 1);
    world.player.x = 8.5;
    for _ in 0..20 {
        world.tick(runtime);
        frame(world, runtime, CENTRE, false, false);
    }
    open_world_map(world, runtime);
    click_text(world, runtime, "Waypoints");
    click_text(world, runtime, "Waypoint 1");
}

#[test]
fn picking_a_row_holds_the_map_on_the_waypoint_while_the_player_walks() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    pick_far_waypoint(&mut world, &runtime);
    assert!(hover(&mut world, &runtime, CENTRE).iter().any(|t| *t == block(40, 8)));
    for _ in 0..64 {
        world.player.x += 1.0;
        world.tick(&runtime);
        frame(&mut world, &runtime, CENTRE, false, false);
    }
    let lines = hover(&mut world, &runtime, CENTRE);
    assert!(lines.iter().any(|t| *t == block(40, 8)), "{lines:?}");
    runtime.shutdown();
}

#[test]
fn a_drag_after_a_pick_moves_the_anchor_and_closing_the_map_clears_it() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    pick_far_waypoint(&mut world, &runtime);
    frame(&mut world, &runtime, (800.0, 400.0), true, false);
    frame(&mut world, &runtime, (768.0, 400.0), true, false);
    frame(&mut world, &runtime, (768.0, 400.0), false, false);
    let lines = hover(&mut world, &runtime, CENTRE);
    assert!(lines.iter().any(|t| *t == block(72, 8)), "{lines:?}");
    key(&mut world, &runtime, ESCAPE, false);
    assert!(!runtime.view_flags().world_map_open, "Escape closes the world map");
    open_world_map(&mut world, &runtime);
    let lines = hover(&mut world, &runtime, CENTRE);
    assert!(lines.iter().any(|t| *t == block(8, 8)), "{lines:?}");
    runtime.shutdown();
}

#[test]
fn hiding_waypoints_keeps_the_list_full() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    add_hotkey_waypoints(&mut world, &runtime, 1);
    open_world_map(&mut world, &runtime);
    hide_waypoints(&mut world, &runtime);
    click_text(&mut world, &runtime, "Waypoints");
    assert!(draws_text(&world.frame, "Waypoint 1"));
    runtime.shutdown();
}

#[test]
fn a_pointer_off_the_window_shows_no_hover() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    world.foreground = false;
    world.draw(&runtime);
    world.draw(&runtime);
    let lines = texts(&world.frame);
    assert!(!lines.iter().any(|t| t.starts_with("X ")), "{lines:?}");
    runtime.shutdown();
}

#[test]
fn the_minimap_reaches_the_mesh_with_its_texture_and_coordinates() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20);
    for _ in 0..3 {
        frame(&mut world, &runtime, CENTRE, false, false);
    }
    assert!(world.frame.mesh.textured().count() > 0, "the corner map draws with its texture");
    assert!(!world.frame.map_open());
    assert!(world.frame.hud.rect.is_positive(), "the hud owns the corner map's rect");
    runtime.shutdown();
}

#[test]
fn a_world_map_frame_owns_the_screen_and_wants_the_arrow() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    frame(&mut world, &runtime, CENTRE, false, false);
    assert!(world.frame.overlay.arrow_cursor);
    assert_eq!(world.frame.overlay.rect.size(), egui::vec2(1280.0, 720.0));
    assert!(world.frame.mesh.text_starting("X "), "the hover lines reach the mesh");
    runtime.shutdown();
}

#[test]
fn typing_in_the_search_filters_the_list() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    add_hotkey_waypoints(&mut world, &runtime, 2);
    open_world_map(&mut world, &runtime);
    click_text(&mut world, &runtime, "Waypoints");
    let listed = |world: &SyntheticWorld| texts(&world.frame).iter().filter(|t| *t == "Waypoint 1").count();
    let before = listed(&world);
    click_text(&mut world, &runtime, "Search waypoints");
    type_text(&mut world, &runtime, "2");
    for _ in 0..3 {
        frame(&mut world, &runtime, CENTRE, false, false);
    }
    let lines = texts(&world.frame);
    assert!(lines.iter().any(|t| t == "Waypoint 2"), "{lines:?}");
    assert_eq!(listed(&world), before - 1, "the search hides the row of Waypoint 1: {lines:?}");
    runtime.shutdown();
}

#[test]
fn a_world_change_drops_the_search_and_a_new_dialog_starts_empty() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    click_text(&mut world, &runtime, "Waypoints");
    click_text(&mut world, &runtime, "Search waypoints");
    type_text(&mut world, &runtime, "q");
    for _ in 0..3 {
        frame(&mut world, &runtime, CENTRE, false, false);
    }
    let typed = texts(&world.frame).iter().any(|t| t == "q");
    runtime.leave_world();
    runtime.enter_world(&world.identity(), 0).unwrap();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20 && rt.waypoints_loaded());
    open_world_map(&mut world, &runtime);
    click_text(&mut world, &runtime, "Waypoints");
    let search_cleared = !texts(&world.frame).iter().any(|t| t == "q");
    menu_pick(&mut world, &runtime, "Add waypoint here");
    assert!(typed && search_cleared);
    assert!(draws_text(&world.frame, "New waypoint"));
    assert!(!texts(&world.frame).iter().any(|t| t == "q"), "the name field starts empty");
    runtime.shutdown();
}

#[test]
fn toolbar_grid_switch_queues_show_chunk_grid() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    let mut out = Vec::new();
    runtime.take_setting_writes(&mut out);

    click_text(&mut world, &runtime, "Chunk grid");
    world.tick(&runtime);
    out.clear();
    runtime.take_setting_writes(&mut out);

    assert_eq!(out, [SettingWrite { path: "world_map.show_chunk_grid", value: SettingValue::Bool(true) }]);
    runtime.shutdown();
}

#[test]
fn the_hud_paints_the_corner_map_and_the_overlay_nothing_while_the_map_is_closed() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.run_until(&runtime, 200, |rt, _| rt.stats().surfaces_resident > 20);

    world.draw(&runtime);

    assert!(world.frame.hud.rect.is_positive() && !world.frame.map_open());
    assert_eq!(world.frame.overlay, minimap_runtime::structs::LayerReply::EMPTY);
    runtime.shutdown();
}

#[test]
fn the_overlay_paints_the_world_map_and_the_hud_nothing_while_it_is_open() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);

    world.draw(&runtime);

    assert!(world.frame.map_open());
    assert_eq!(world.frame.hud, minimap_runtime::structs::LayerReply::EMPTY);
    assert!(draws_text(&world.frame, "Chunk grid"));
    runtime.shutdown();
}

#[test]
fn a_toast_shows_on_the_hud_when_closed_and_on_the_overlay_when_open() {
    let mut world = SyntheticWorld::new();
    let runtime = world.runtime();
    world.tick(&runtime);
    bind(&runtime, KeybindAction::ToggleCave, true);
    world.draw(&runtime);
    let on_hud = draws_text(&world.frame, "Cave view on") && !world.frame.map_open();

    open_world_map(&mut world, &runtime);
    bind(&runtime, KeybindAction::ToggleCave, false);
    world.draw(&runtime);

    assert!(on_hud);
    assert!(world.frame.map_open() && draws_text(&world.frame, "Cave view off"), "{:?}", texts(&world.frame));
    runtime.shutdown();
}

// frames with the world map open until a region tile is uploaded; true when one was
fn wait_for_region_upload(world: &mut SyntheticWorld, runtime: &Runtime) -> bool {
    for _ in 0..300 {
        world.tick(runtime);
        world.draw(runtime);
        if region_uploads(&world.frame) > 0 {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
    false
}

#[test]
fn a_new_overlay_context_uploads_the_visible_tiles_again() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    assert!(wait_for_region_upload(&mut world, &runtime));
    for _ in 0..30 {
        world.draw(&runtime);
    }

    world.overlay.replace_context();

    assert!(wait_for_region_upload(&mut world, &runtime), "the tiles go to the new context");
    runtime.shutdown();
}

#[test]
fn textures_lost_uploads_the_visible_tiles_and_the_corner_map_again() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    assert!(wait_for_region_upload(&mut world, &runtime));
    for _ in 0..30 {
        world.draw(&runtime);
    }

    runtime.textures_lost();

    assert!(wait_for_region_upload(&mut world, &runtime), "the region tiles are uploaded again");
    runtime.shutdown();
}

#[test]
fn zoomed_out_tiles_stay_inside_a_small_grant() {
    let mut world = SyntheticWorld::new();
    world.grant = TextureGrant { bytes: 16 * 1024 * 1024 + 512 * 1024, count: 4096 };
    let runtime = explored_map(&mut world);
    assert!(world.run_until(&runtime, 500, |rt, _| rt.stats().tiles_resident > 2), "{:?}", runtime.stats());

    for _ in 0..100 {
        world.paint(&runtime, CENTRE, false, false, &[HarnessEvent::Wheel(-120)]);
        assert!(runtime.stats().gpu_textures <= 3, "{:?}", runtime.stats());
    }

    runtime.shutdown();
}

#[test]
fn a_keybind_while_typing_in_the_search_does_not_close_the_map() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    click_text(&mut world, &runtime, "Waypoints");
    click_text(&mut world, &runtime, "Search waypoints");
    type_text(&mut world, &runtime, "m");

    let toggled = bind(&runtime, KeybindAction::ToggleWorldMap, false);
    let cave = bind(&runtime, KeybindAction::ToggleCave, false);

    assert!(runtime.view_flags().world_map_open, "the typed M does not close the map");
    assert_eq!(toggled, KeyResult::default());
    assert!(!cave.toggle_cave, "the typed K does not switch cave view");
    runtime.shutdown();
}

#[test]
fn a_keybind_closes_the_map_again_once_the_search_lets_go_of_the_keyboard() {
    let mut world = SyntheticWorld::new();
    let runtime = explored_map(&mut world);
    click_text(&mut world, &runtime, "Waypoints");
    click_text(&mut world, &runtime, "Search waypoints");
    type_text(&mut world, &runtime, "m");
    click(&mut world, &runtime, CENTRE, false);
    frame(&mut world, &runtime, CENTRE, false, false);

    bind(&runtime, KeybindAction::ToggleWorldMap, false);

    assert!(!runtime.view_flags().world_map_open);
    runtime.shutdown();
}
