use minimap_core::config::Config;
use minimap_core::services::draw::WaypointPainter;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::Rgba;
use minimap_core::structs::draw::{DrawCommand, DrawList, MinimapFrame, WorldMapFrame};
use minimap_core::structs::view::CameraPose;
use minimap_core::structs::waypoint::{Waypoint, WaypointSource};

const DISC: ((f32, f32), f32) = ((1000.0, 120.0), 100.0);

fn minimap(angle: f32) -> MinimapFrame {
    MinimapFrame {
        display_width: 1200.0,
        camera: CameraPose { x: 0.5, z: 0.5, yaw: 0.0 },
        player_block: (0, 70, 0),
        radius_chunks: 5,
        font_size: 18.0,
        texture: None,
        image_origin: (0, 0),
        image_side: 162,
        dim: 0,
        show_coordinates: true,
        angle,
    }
}

fn world_map(zoom: f32) -> WorldMapFrame<'static> {
    WorldMapFrame {
        display: (1280.0, 720.0),
        centre: (0.5, 0.5),
        zoom,
        camera: CameraPose { x: 0.5, z: 0.5, yaw: 0.0 },
        player_block: (0, 70, 0),
        font_size: 18.0,
        show_coordinates: true,
        show_nether_coordinates: true,
        dim: 0,
        angle: 0.0,
        regions: &[],
    }
}

// waypoints at (x, z) in the Overworld, named as given
fn waypoints(places: &[(&str, i32, i32)]) -> Vec<Waypoint> {
    let mut store = WaypointStore::new(64, 3);
    for &(name, x, z) in places {
        store.add(name, 0, (x, 64, z), WaypointSource::Manual, 0).unwrap();
    }
    store.waypoints().to_vec()
}

fn texts(draw: &DrawList) -> Vec<(String, (f32, f32))> {
    let mut out: Vec<(String, (f32, f32))> = draw
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { start, len, pos, colour, .. } if *colour == 0xFFFF_FFFF => {
                Some((draw.text_of(*start, *len).to_string(), *pos))
            }
            _ => None,
        })
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out
}

// tips of the coloured triangles that touch the minimap rim
fn arrow_tips(draw: &DrawList, colour: [u8; 4]) -> Vec<(f32, f32)> {
    let packed = Rgba::new(colour[0], colour[1], colour[2], colour[3]).packed();
    let (centre, radius) = DISC;
    draw.commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::TriangleFilled { a, colour, .. } if *colour == packed => Some(*a),
            _ => None,
        })
        .filter(|a| ((a.0 - centre.0).hypot(a.1 - centre.1) - radius).abs() < 0.01)
        .collect()
}

fn paint_minimap(config: &Config, frame: &MinimapFrame, list: &[Waypoint]) -> DrawList {
    let mut draw = DrawList::new();
    WaypointPainter::new(config).paint_minimap(frame, DISC, list, &mut draw);
    draw
}

#[test]
fn a_waypoint_inside_the_minimap_is_a_diamond_with_its_name_above() {
    let list = waypoints(&[("Home", 16, 0)]);
    let draw = paint_minimap(&Config::default(), &minimap(0.0), &list);
    let triangles = draw.commands.iter().filter(|c| matches!(c, DrawCommand::TriangleFilled { .. })).count();
    assert_eq!(triangles, 4);
    let labels = texts(&draw);
    assert_eq!(labels.len(), 1);
    assert_eq!(labels[0].0, "Home");
    // 16 blocks east at 100 px per 80 blocks
    assert!((labels[0].1.0 - 1020.0).abs() < 1e-3);
    assert!(labels[0].1.1 < DISC.0.1 - WaypointPainter::ICON);
}

#[test]
fn only_the_frames_dimension_is_drawn() {
    let mut list = waypoints(&[("Home", 16, 0)]);
    list[0].dim = -1;
    assert!(paint_minimap(&Config::default(), &minimap(0.0), &list).commands.is_empty());
}

#[test]
fn a_far_waypoint_gets_an_arrow_on_the_rim_pointing_at_it_and_its_distance() {
    let list = waypoints(&[("Far", 300, 0)]);
    let draw = paint_minimap(&Config::default(), &minimap(0.0), &list);
    let tips = arrow_tips(&draw, list[0].colour);
    assert_eq!(tips.len(), 1);
    assert!((tips[0].0 - 1100.0).abs() < 1e-3 && (tips[0].1 - 120.0).abs() < 1e-3);
    assert_eq!(texts(&draw)[0].0, "Far 300 m");

    let mut quiet = list.clone();
    quiet[0].show_distance = false;
    assert_eq!(texts(&paint_minimap(&Config::default(), &minimap(0.0), &quiet))[0].0, "Far");
    quiet[0].show_on_edge = false;
    assert!(paint_minimap(&Config::default(), &minimap(0.0), &quiet).commands.is_empty());
}

#[test]
fn edge_arrows_turn_with_a_player_up_minimap() {
    let list = waypoints(&[("Far", 300, 0)]);
    let draw = paint_minimap(&Config::default(), &minimap(std::f32::consts::FRAC_PI_2), &list);
    let tips = arrow_tips(&draw, list[0].colour);
    assert!((tips[0].0 - 1000.0).abs() < 1e-3 && (tips[0].1 - 220.0).abs() < 1e-3, "{tips:?}");
}

#[test]
fn edge_arrows_are_capped_and_the_nearest_are_kept() {
    let places: Vec<(String, i32, i32)> = (0..20).map(|i| (format!("W{i:02}"), 200 + i * 10, i * 7)).collect();
    let refs: Vec<(&str, i32, i32)> = places.iter().map(|(n, x, z)| (n.as_str(), *x, *z)).collect();
    let list = waypoints(&refs);
    let draw = paint_minimap(&Config::default(), &minimap(0.0), &list);
    let names: Vec<String> = texts(&draw).into_iter().map(|(t, _)| t.split(' ').next().unwrap().to_string()).collect();
    assert_eq!(names.len(), 16);
    assert!(names.contains(&"W00".to_string()) && names.contains(&"W15".to_string()));
    assert!(!names.contains(&"W16".to_string()) && !names.contains(&"W19".to_string()));

    let mut few = Config::default();
    few.waypoints.max_edge_arrows = 2;
    assert_eq!(texts(&paint_minimap(&few, &minimap(0.0), &list)).len(), 2);
}

#[test]
fn labels_that_would_overlap_are_stacked_a_line_apart_upward() {
    let list = waypoints(&[("Alpha", 16, 0), ("Bravo", 16, 0)]);
    let labels = texts(&paint_minimap(&Config::default(), &minimap(0.0), &list));
    let size = 18.0;
    assert!((labels[0].1.1 - labels[1].1.1 - size * 1.25).abs() < 1e-3, "{labels:?}");
}

#[test]
fn the_death_point_is_drawn_as_a_skull() {
    let mut store = WaypointStore::new(4, 1);
    store.set_death(0, (8, 64, 0), 0);
    let draw = paint_minimap(&Config::default(), &minimap(0.0), store.waypoints());
    let circles = draw.commands.iter().filter(|c| matches!(c, DrawCommand::CircleFilled { .. })).count();
    assert_eq!(circles, 4);
    assert!(!draw.commands.iter().any(|c| matches!(c, DrawCommand::TriangleFilled { .. })));
}

#[test]
fn the_world_map_draws_markers_in_view_and_names_from_the_label_zoom() {
    let list = waypoints(&[("Near", 100, 0), ("Gone", 100_000, 0)]);
    let mut painter = WaypointPainter::new(&Config::default());
    let mut draw = DrawList::new();
    painter.paint_world_map(&world_map(1.0), &list, &mut draw);
    assert_eq!(draw.commands.iter().filter(|c| matches!(c, DrawCommand::TriangleFilled { .. })).count(), 4);
    let labels = texts(&draw);
    assert_eq!(labels.len(), 1);
    assert!((labels[0].1.0 - 740.0).abs() < 1e-3);
    draw.clear();
    painter.paint_world_map(&world_map(0.25), &list, &mut draw);
    assert!(texts(&draw).is_empty());
    assert_eq!(draw.commands.iter().filter(|c| matches!(c, DrawCommand::TriangleFilled { .. })).count(), 4);
}

#[test]
fn with_waypoints_turned_off_nothing_is_drawn_on_either_map() {
    let mut config = Config::default();
    config.waypoints.enabled = false;
    let list = waypoints(&[("Home", 16, 0), ("Far", 300, 0)]);
    assert!(paint_minimap(&config, &minimap(0.0), &list).commands.is_empty());
    let mut draw = DrawList::new();
    WaypointPainter::new(&config).paint_world_map(&world_map(1.0), &list, &mut draw);
    assert!(draw.commands.is_empty());
    config.waypoints.enabled = true;
    config.waypoints.show_on_minimap = false;
    assert!(paint_minimap(&config, &minimap(0.0), &list).commands.is_empty());
}

#[test]
fn a_right_click_hits_the_nearest_marker_within_reach_in_the_dimension() {
    let mut list = waypoints(&[("Near", 100, 0), ("Nearer", 102, 0), ("Nether", 100, 0)]);
    list[2].dim = -1;
    let painter = WaypointPainter::new(&Config::default());
    let frame = world_map(1.0);
    assert_eq!(painter.hit(&frame, &list, (742.0, 360.0)), Some(list[1].id));
    assert_eq!(painter.hit(&frame, &list, (738.0, 360.0)), Some(list[0].id));
    assert_eq!(painter.hit(&frame, &list, (760.0, 360.0)), None);
}

#[test]
fn labels_are_drawn_at_the_text_size_times_the_label_scale_on_a_dark_band() {
    let list = waypoints(&[("Home", 16, 0)]);
    let mut config = Config::default();
    config.waypoints.label_scale = 2.0;
    let draw = paint_minimap(&config, &minimap(0.0), &list);
    let text = draw.commands.iter().find_map(|c| match c {
        DrawCommand::Text { size, .. } => Some(*size),
        _ => None,
    });
    assert_eq!(text, Some(36.0));
    let first = draw.commands.iter().position(|c| matches!(c, DrawCommand::Text { .. })).unwrap();
    assert!(matches!(draw.commands[first - 1], DrawCommand::RectFilled { .. }));
}

#[test]
fn markers_grow_with_the_frame_text_size() {
    let list = waypoints(&[("Home", 16, 0)]);
    let mut big = minimap(0.0);
    big.font_size = 36.0;
    let top_y = |draw: &DrawList| {
        draw.commands.iter().find_map(|c| match c {
            DrawCommand::TriangleFilled { a, .. } => Some(a.1),
            _ => None,
        })
    };
    let normal = top_y(&paint_minimap(&Config::default(), &minimap(0.0), &list)).unwrap();
    let doubled = top_y(&paint_minimap(&Config::default(), &big, &list)).unwrap();
    // the outline diamond's top sits (icon + outline) above the marker centre at y 120
    assert!(((120.0 - doubled) - 2.0 * (120.0 - normal)).abs() < 1e-3, "{normal} {doubled}");
}

#[test]
fn an_edge_label_is_pulled_in_so_it_stays_inside_the_rim() {
    let list = waypoints(&[("Jens House", 300, 0)]);
    let draw = paint_minimap(&Config::default(), &minimap(0.0), &list);
    let (centre, radius) = DISC;
    let band = draw.commands.iter().find_map(|c| match c {
        DrawCommand::RectFilled { p1, .. } => Some(*p1),
        _ => None,
    });
    assert!(band.unwrap().0 <= centre.0 + radius, "{band:?}");
}

#[test]
fn a_waypoint_under_another_marker_is_found_by_a_right_click_on_its_name() {
    let list = waypoints(&[("Waypoint 1", 100, 0), ("Waypoint 2", 100, 0)]);
    let mut painter = WaypointPainter::new(&Config::default());
    let frame = world_map(1.0);
    let mut draw = DrawList::new();
    painter.paint_world_map(&frame, &list, &mut draw);
    let tops: Vec<(String, (f32, f32))> = texts(&draw);
    assert_eq!(painter.hit(&frame, &list, (740.0, 360.0)), Some(list[0].id));
    let second = tops.iter().find(|(name, _)| name == "Waypoint 2").unwrap().1;
    assert_eq!(painter.hit(&frame, &list, (second.0, second.1 + 5.0)), Some(list[1].id));
}
