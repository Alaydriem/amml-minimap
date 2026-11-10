use egui::epaint::Mesh;
use minimap_core::structs::draw::{DrawCommand, DrawList, SamplerKind, TextAlign};

use crate::support::Frames;

fn image(tex: u32) -> DrawCommand {
    DrawCommand::Image { tex, p0: (10.0, 10.0), p1: (266.0, 266.0), uv0: (0.0, 0.0), uv1: (1.0, 1.0) }
}

fn only_textured(list: &DrawList, textures: &[u32]) -> Mesh {
    let (out, _) = Frames::paint(list, textures, 1.0);
    let meshes: Vec<&Mesh> = out.textured().collect();
    assert_eq!(meshes.len(), 1, "one textured mesh");
    meshes[0].clone()
}

#[test]
fn a_map_image_draws_with_its_layer_texture() {
    let mut list = DrawList::new();
    list.push(image(3));
    list.push(DrawCommand::Sampler(SamplerKind::Linear));
    list.push(image(7));

    let (out, layer) = Frames::paint(&list, &[3, 7], 1.0);

    let ids: Vec<_> = out.textured().map(|mesh| mesh.texture_id).collect();
    assert_eq!(ids, [layer.id(3).unwrap(), layer.id(7).unwrap()]);
}

#[test]
fn an_image_whose_texture_the_layer_does_not_hold_is_left_out() {
    let mut list = DrawList::new();
    list.push(image(3));
    list.push(image(5));

    let (out, _) = Frames::paint(&list, &[3], 1.0);

    assert_eq!(out.textured().count(), 1);
}

#[test]
fn an_image_disc_stays_inside_its_circle() {
    let mut list = DrawList::new();
    list.push(DrawCommand::ImageDisc {
        tex: 1,
        centre: (200.0, 200.0),
        radius: 100.0,
        uv_centre: (0.5, 0.5),
        uv_radius: 0.4,
        angle: 0.6,
    });

    let mesh = only_textured(&list, &[1]);

    assert!(mesh.indices.len() / 3 >= 32);
    assert!(mesh.vertices.iter().all(|v| ((v.pos.x - 200.0).hypot(v.pos.y - 200.0)) <= 100.5));
    assert!(mesh.vertices.iter().all(|v| ((v.uv.x - 0.5).hypot(v.uv.y - 0.5)) <= 0.4001));
}

#[test]
fn a_turned_image_keeps_its_corner_uvs_and_turns_its_corners() {
    let mut list = DrawList::new();
    list.push(DrawCommand::ImageRotated {
        tex: 4,
        centre: (300.0, 300.0),
        half: (100.0, 50.0),
        uv0: (0.0, 0.0),
        uv1: (1.0, 1.0),
        angle: std::f32::consts::FRAC_PI_2,
    });

    let mesh = only_textured(&list, &[4]);

    let corner = mesh.vertices.iter().find(|v| v.uv.x == 0.0 && v.uv.y == 0.0).map(|v| v.pos);
    let corner = corner.unwrap_or_default();
    assert!((corner.x - 350.0).abs() < 0.01 && (corner.y - 200.0).abs() < 0.01, "{corner:?}");
    for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
        assert!(mesh.vertices.iter().any(|vertex| (vertex.uv.x, vertex.uv.y) == (u, v)));
    }
}

#[test]
fn text_commands_are_laid_out_with_their_string() {
    let mut list = DrawList::new();
    list.text((640.0, 20.0), 0xFFFF_FFFF, 18.0, TextAlign::Centre, "12, 64, -3");

    let (out, _) = Frames::paint(&list, &[], 1.0);

    let rect = out.text_rect("12, 64, -3").unwrap_or_default();
    assert!(((rect[0] + rect[2]) * 0.5 - 640.0).abs() < 1.0 && rect[1] >= 19.0, "{rect:?}");
}

#[test]
fn map_shapes_divide_physical_pixels_by_the_pixels_per_point() {
    let mut list = DrawList::new();
    list.push(image(9));

    let (out, _) = Frames::paint(&list, &[9], 2.0);

    let xs: Vec<f32> = out.textured().flat_map(|mesh| mesh.vertices.iter().map(|v| v.pos.x)).collect();
    assert!(!xs.is_empty());
    assert!(xs.iter().all(|x| (x - 5.0).abs() < 0.01 || (x - 133.0).abs() < 0.01), "{xs:?}");
}
