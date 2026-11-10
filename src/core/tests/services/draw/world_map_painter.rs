use minimap_core::services::draw::WorldMapPainter;
use minimap_core::structs::draw::{DrawCommand, DrawList, SamplerKind, TextAlign, WorldMapFrame};
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::view::CameraPose;

fn paint(zoom: f32, show_coordinates: bool, regions: &[(RegionPos, Option<u32>)]) -> DrawList {
    paint_turned(zoom, show_coordinates, regions, 0.0)
}

fn paint_turned(zoom: f32, show_coordinates: bool, regions: &[(RegionPos, Option<u32>)], angle: f32) -> DrawList {
    let frame = WorldMapFrame {
        display: (800.0, 600.0),
        centre: (128.0, 128.0),
        zoom,
        camera: CameraPose { x: 128.0, z: 128.0, yaw: 0.0 },
        player_block: (128, 64, 128),
        font_size: 18.0,
        show_coordinates,
        show_nether_coordinates: false,
        dim: 0,
        angle,
        regions,
    };
    let mut draw = DrawList::new();
    WorldMapPainter::paint(&frame, &mut draw);
    draw
}

#[test]
fn resident_regions_are_images_and_others_placeholders() {
    let regions = [(RegionPos::new(0, 0, 0), Some(7)), (RegionPos::new(1, 0, 0), None)];
    let draw = paint(1.0, true, &regions);
    let image = draw.commands.iter().find_map(|c| match c {
        DrawCommand::Image { tex, p0, p1, .. } => Some((*tex, *p0, *p1)),
        _ => None,
    });
    assert_eq!(image, Some((7, (272.0, 172.0), (528.0, 428.0))));
    let rects = draw.commands.iter().filter(|c| matches!(c, DrawCommand::RectFilled { .. })).count();
    assert_eq!(rects, 2);
}

#[test]
fn zoom_below_one_samples_linearly_and_at_or_above_one_by_point() {
    let near = paint(2.0, true, &[]);
    assert_eq!(near.commands[1], DrawCommand::Sampler(SamplerKind::Point));
    assert!(near.commands.contains(&DrawCommand::Sampler(SamplerKind::Linear)));
    let far = paint(0.5, true, &[]);
    assert_eq!(far.commands[1], DrawCommand::Sampler(SamplerKind::Linear));
}

#[test]
fn the_bottom_text_reads_in_english_and_can_be_turned_off() {
    let draw = paint(1.25, true, &[]);
    let text = draw.commands.iter().rev().find_map(|c| match c {
        DrawCommand::Text { start, len, align, .. } => Some((draw.text_of(*start, *len).to_string(), *align)),
        _ => None,
    });
    assert_eq!(text, Some(("X: 128  Z: 128   Zoom: 1.25x".to_string(), TextAlign::CentreBottom)));
    assert!(!paint(1.0, false, &[]).commands.iter().any(|c| matches!(c, DrawCommand::Text { .. })));
}

#[test]
fn a_turned_map_draws_tiles_as_turned_quads_around_the_screen_centre() {
    let regions = [(RegionPos::new(0, 0, 0), Some(7)), (RegionPos::new(1, 0, 0), None)];
    let angle = std::f32::consts::FRAC_PI_2;
    let draw = paint_turned(1.0, true, &regions, angle);
    assert!(!draw.commands.iter().any(|c| matches!(c, DrawCommand::Image { .. })));
    let quad = draw.commands.iter().find_map(|c| match c {
        DrawCommand::ImageRotated { tex, centre, half, angle, .. } => Some((*tex, *centre, *half, *angle)),
        _ => None,
    });
    // region 0 sits on the screen centre at zoom 1, so turning it keeps it there
    let (tex, centre, half, turned) = quad.expect("a turned tile");
    assert_eq!((tex, half, turned), (7, (128.0, 128.0), angle));
    assert!((centre.0 - 400.0).abs() < 1e-3 && (centre.1 - 300.0).abs() < 1e-3);
    let triangles = draw.commands.iter().filter(|c| matches!(c, DrawCommand::TriangleFilled { .. })).count();
    // two for the placeholder, two for the player arrow
    assert_eq!(triangles, 4);
}

#[test]
fn the_bottom_text_adds_the_nether_position_when_asked() {
    let frame = WorldMapFrame {
        display: (800.0, 600.0),
        centre: (128.0, 128.0),
        zoom: 1.0,
        camera: CameraPose { x: 128.0, z: 128.0, yaw: 0.0 },
        player_block: (-20, 64, 128),
        font_size: 18.0,
        show_coordinates: true,
        show_nether_coordinates: true,
        dim: 0,
        angle: 0.0,
        regions: &[],
    };
    let mut draw = DrawList::new();
    WorldMapPainter::paint(&frame, &mut draw);
    let text = draw.commands.iter().rev().find_map(|c| match c {
        DrawCommand::Text { start, len, .. } => Some(draw.text_of(*start, *len).to_string()),
        _ => None,
    });
    assert_eq!(text.as_deref(), Some("X: -20  Z: 128   Nether -3, 16   Zoom: 1.00x"));
}
