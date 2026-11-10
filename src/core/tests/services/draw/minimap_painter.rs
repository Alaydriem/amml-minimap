use minimap_core::config::MiniMapConfig;
use minimap_core::services::draw::MinimapPainter;
use minimap_core::structs::draw::{DrawCommand, DrawList, MinimapFrame, TextAlign};
use minimap_core::structs::view::CameraPose;

fn frame(texture: Option<u32>) -> MinimapFrame {
    MinimapFrame {
        display_width: 1200.0,
        camera: CameraPose { x: 8.5, z: 8.5, yaw: 0.0 },
        player_block: (8, 70, 8),
        radius_chunks: 5,
        font_size: 18.0,
        texture,
        image_origin: (-73, -73),
        image_side: 162,
        dim: 0,
        show_coordinates: true,
        angle: 0.0,
    }
}

fn paint(config: &MiniMapConfig, texture: Option<u32>) -> DrawList {
    let mut draw = DrawList::new();
    MinimapPainter::new(config).paint(&frame(texture), &mut draw);
    draw
}

#[test]
fn the_disc_sits_in_the_top_right_corner_and_scales_with_the_widget() {
    let painter = MinimapPainter::new(&sixth());
    let (centre, radius) = painter.disc(1200.0);
    assert!((radius - 100.0).abs() < 1e-3);
    assert!((centre.0 - 1080.0).abs() < 1e-3 && (centre.1 - 120.0).abs() < 1e-3);
    let scaled = MinimapPainter::new(&MiniMapConfig { widget_scale: 2.0, ..sixth() });
    let (centre, radius) = scaled.disc(1200.0);
    assert!((radius - 200.0).abs() < 1e-3);
    assert!((centre.0 - 960.0).abs() < 1e-3 && (centre.1 - 240.0).abs() < 1e-3);
}

#[test]
fn the_terrain_texture_is_a_disc_the_size_of_the_border_ring_with_uvs_around_the_camera() {
    let wide = MiniMapConfig { widget_scale: 4.0, ..sixth() };
    let draw = paint(&wide, Some(1));
    let image = draw.commands.iter().find_map(|c| match c {
        DrawCommand::ImageDisc { tex, centre, radius, uv_centre, uv_radius, angle } => {
            Some((*tex, *centre, *radius, *uv_centre, *uv_radius, *angle))
        }
        _ => None,
    });
    let ring = draw.commands.iter().find_map(|c| match c {
        DrawCommand::Circle { centre, radius, .. } => Some((*centre, *radius)),
        _ => None,
    });
    let (tex, centre, radius, uv_centre, uv_radius, angle) = image.expect("an image");
    assert_eq!(tex, 1);
    assert_eq!(angle, 0.0);
    assert_eq!(Some((centre, radius)), ring);
    assert!((radius - 400.0).abs() < 1e-3);
    assert!((uv_centre.0 - (8.5 + 73.0) / 162.0).abs() < 1e-5);
    assert!((uv_radius - 80.0 / 162.0).abs() < 1e-5);
    let no_texture = paint(&sixth(), None);
    assert!(!no_texture.commands.iter().any(|c| matches!(c, DrawCommand::ImageDisc { .. })));
}

#[test]
fn coordinates_are_drawn_below_with_a_shadow_unless_turned_off() {
    let draw = paint(&sixth(), None);
    let texts: Vec<_> = draw
        .commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { start, len, align, size, .. } => {
                Some((draw.text_of(*start, *len).to_string(), *align, *size))
            }
            _ => None,
        })
        .collect();
    assert_eq!(texts.len(), 4);
    assert_eq!(texts[1].0, "8, 70, 8");
    assert_eq!(texts[1].1, TextAlign::Centre);
    assert_eq!(texts[3].0, "Nether 1, 1");
    let mut hidden = DrawList::new();
    MinimapPainter::new(&sixth()).paint(&MinimapFrame { show_coordinates: false, ..frame(None) }, &mut hidden);
    assert!(!hidden.commands.iter().any(|c| matches!(c, DrawCommand::Text { .. })));
}

#[test]
fn widget_scale_multiplies_line_widths_and_text_size() {
    let config = MiniMapConfig { widget_scale: 1.5, ..sixth() };
    let draw = paint(&config, None);
    assert!(
        draw.commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Line { thickness, .. } if (*thickness - 1.5).abs() < 1e-6))
    );
    assert!(
        draw.commands
            .iter()
            .any(|c| matches!(c, DrawCommand::Circle { thickness, .. } if (*thickness - 3.0).abs() < 1e-6))
    );
    assert!(draw.commands.iter().any(|c| matches!(c, DrawCommand::Text { size, .. } if (*size - 27.0).abs() < 1e-6)));
}

#[test]
fn every_grid_line_stays_inside_the_disc() {
    let draw = paint(&sixth(), None);
    for command in &draw.commands {
        if let DrawCommand::Line { a, b, .. } = command {
            for p in [a, b] {
                let d = ((p.0 - 1080.0).powi(2) + (p.1 - 120.0).powi(2)).sqrt();
                assert!(d <= 99.01, "{p:?} is {d} from the centre");
            }
        }
    }
}

fn texts(draw: &DrawList) -> Vec<String> {
    draw.commands
        .iter()
        .filter_map(|c| match c {
            DrawCommand::Text { start, len, .. } => Some(draw.text_of(*start, *len).to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_nether_line_follows_its_own_switch_and_the_dimension() {
    let off = MiniMapConfig { show_nether_coordinates: false, ..sixth() };
    assert_eq!(texts(&paint(&off, None)), vec!["8, 70, 8".to_string(), "8, 70, 8".to_string()]);
    let mut nether = DrawList::new();
    MinimapPainter::new(&sixth()).paint(&MinimapFrame { dim: 1, ..frame(None) }, &mut nether);
    assert!(texts(&nether).contains(&"Overworld 64, 64".to_string()));
    let mut end = DrawList::new();
    MinimapPainter::new(&sixth()).paint(&MinimapFrame { dim: 2, ..frame(None) }, &mut end);
    assert_eq!(texts(&end).len(), 2);
}

#[test]
fn a_turned_minimap_draws_a_disc_centred_on_the_camera() {
    let mut draw = DrawList::new();
    MinimapPainter::new(&sixth()).paint(&MinimapFrame { angle: 0.6, ..frame(Some(1)) }, &mut draw);
    let disc = draw.commands.iter().find_map(|c| match c {
        DrawCommand::ImageDisc { tex, centre, radius, uv_centre, uv_radius, angle } => {
            Some((*tex, *centre, *radius, *uv_centre, *uv_radius, *angle))
        }
        _ => None,
    });
    let (tex, centre, radius, uv_centre, uv_radius, angle) = disc.expect("a disc");
    assert_eq!((tex, centre, angle), (1, (1080.0, 120.0), 0.6));
    assert!((radius - 100.0).abs() < 1e-3);
    assert!((uv_centre.0 - (8.5 + 73.0) / 162.0).abs() < 1e-5);
    assert!((uv_radius - 80.0 / 162.0).abs() < 1e-5);
}

#[test]
fn turned_grid_lines_still_stay_inside_the_disc() {
    for angle in [0.3f32, 1.2, 2.5, -0.8] {
        let mut draw = DrawList::new();
        MinimapPainter::new(&sixth()).paint(&MinimapFrame { angle, ..frame(None) }, &mut draw);
        let lines = draw.commands.iter().filter(|c| matches!(c, DrawCommand::Line { .. })).count();
        assert!(lines > 10);
        for command in &draw.commands {
            if let DrawCommand::Line { a, b, .. } = command {
                for p in [a, b] {
                    let d = ((p.0 - 1080.0).powi(2) + (p.1 - 120.0).powi(2)).sqrt();
                    assert!(d <= 99.01, "angle {angle}: {p:?} is {d} from the centre");
                }
            }
        }
    }
}

// the disc at a sixth of the screen width, the size these geometry checks were measured at
fn sixth() -> MiniMapConfig {
    MiniMapConfig { size_ratio: 1.0 / 6.0, ..MiniMapConfig::default() }
}
