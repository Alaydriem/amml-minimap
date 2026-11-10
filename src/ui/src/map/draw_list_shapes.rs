use egui::epaint::{Mesh, Vertex};
use egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, TextureId, pos2};
use minimap_core::structs::draw::{DrawCommand, DrawList, TextAlign};

use crate::map::LayerTextures;

// turns the map painters' draw list (physical pixels, IM_COL32 colours) into egui shapes; an image whose
// texture the layer does not hold yet is left out
pub struct DrawListShapes;

impl DrawListShapes {
    // segments of a minimap disc, as the shim drew it before
    pub const DISC_SEGMENTS: usize = 64;
    const UV_CORNERS: [(usize, usize); 4] = [(0, 0), (1, 0), (1, 1), (0, 1)];

    pub fn new() -> Self {
        Self
    }

    pub fn paint(&mut self, list: &DrawList, painter: &Painter, pixels_per_point: f32, textures: &LayerTextures) {
        let scale = 1.0 / pixels_per_point.max(f32::EPSILON);
        let at = |p: (f32, f32)| pos2(p.0 * scale, p.1 * scale);
        for command in &list.commands {
            match *command {
                DrawCommand::RectFilled { p0, p1, colour } => {
                    painter.rect_filled(Rect::from_two_pos(at(p0), at(p1)), 0, Self::colour(colour));
                }
                DrawCommand::Line { a, b, colour, thickness } => {
                    painter.line_segment([at(a), at(b)], Stroke::new(thickness * scale, Self::colour(colour)));
                }
                DrawCommand::TriangleFilled { a, b, c, colour } => {
                    painter.add(Shape::convex_polygon(vec![at(a), at(b), at(c)], Self::colour(colour), Stroke::NONE));
                }
                DrawCommand::CircleFilled { centre, radius, colour } => {
                    painter.circle_filled(at(centre), radius * scale, Self::colour(colour));
                }
                DrawCommand::Circle { centre, radius, colour, thickness } => {
                    painter.circle_stroke(
                        at(centre),
                        radius * scale,
                        Stroke::new(thickness * scale, Self::colour(colour)),
                    );
                }
                DrawCommand::Text { pos, colour, size, align, start, len } => {
                    let anchor = match align {
                        TextAlign::Left => Align2::LEFT_TOP,
                        TextAlign::Centre => Align2::CENTER_TOP,
                        TextAlign::CentreBottom => Align2::CENTER_BOTTOM,
                    };
                    let text = list.text_of(start, len);
                    painter.text(at(pos), anchor, text, FontId::proportional(size * scale), Self::colour(colour));
                }
                DrawCommand::Image { tex, p0, p1, uv0, uv1 } => {
                    let Some(id) = textures.id(tex) else { continue };
                    let uv = Rect::from_min_max(pos2(uv0.0, uv0.1), pos2(uv1.0, uv1.1));
                    painter.image(id, Rect::from_two_pos(at(p0), at(p1)), uv, Color32::WHITE);
                }
                DrawCommand::ImageRotated { tex, centre, half, uv0, uv1, angle } => {
                    let Some(id) = textures.id(tex) else { continue };
                    painter.add(Self::turned(id, at(centre), (half.0 * scale, half.1 * scale), (uv0, uv1), angle));
                }
                DrawCommand::ImageDisc { tex, centre, radius, uv_centre, uv_radius, angle } => {
                    let Some(id) = textures.id(tex) else { continue };
                    painter.add(Self::disc(id, at(centre), radius * scale, uv_centre, uv_radius, angle));
                }
                // the sampler belongs to the texture now
                DrawCommand::Sampler(_) => {}
            }
        }
    }

    // IM_COL32 packing: r in the low byte, a in the high byte
    pub fn colour(packed: u32) -> Color32 {
        let [r, g, b, a] = packed.to_le_bytes();
        Color32::from_rgba_unmultiplied(r, g, b, a)
    }

    // an axis-aligned image turned by `angle` around `centre`; corners keep their uv corners
    fn turned(id: TextureId, centre: Pos2, half: (f32, f32), uv: ((f32, f32), (f32, f32)), angle: f32) -> Shape {
        let (s, c) = angle.sin_cos();
        let us = [uv.0.0, uv.1.0];
        let vs = [uv.0.1, uv.1.1];
        let mut mesh = Mesh::with_texture(id);
        for (ux, vy) in Self::UV_CORNERS {
            let x = if ux == 0 { -half.0 } else { half.0 };
            let y = if vy == 0 { -half.1 } else { half.1 };
            let pos = pos2(centre.x + x * c - y * s, centre.y + x * s + y * c);
            mesh.vertices.push(Vertex { pos, uv: pos2(us[ux], vs[vy]), color: Color32::WHITE });
        }
        mesh.add_triangle(0, 1, 2);
        mesh.add_triangle(0, 2, 3);
        Shape::mesh(mesh)
    }

    // a triangle fan; each rim vertex samples the texture circle turned back by `angle`
    fn disc(id: TextureId, centre: Pos2, radius: f32, uv_centre: (f32, f32), uv_radius: f32, angle: f32) -> Shape {
        let (s, c) = (-angle).sin_cos();
        let mut mesh = Mesh::with_texture(id);
        mesh.reserve_vertices(Self::DISC_SEGMENTS + 1);
        mesh.reserve_triangles(Self::DISC_SEGMENTS);
        mesh.vertices.push(Vertex { pos: centre, uv: pos2(uv_centre.0, uv_centre.1), color: Color32::WHITE });
        for i in 0..Self::DISC_SEGMENTS {
            let a = std::f32::consts::TAU * i as f32 / Self::DISC_SEGMENTS as f32;
            let (dy, dx) = a.sin_cos();
            let (ux, uy) = (dx * c - dy * s, dx * s + dy * c);
            mesh.vertices.push(Vertex {
                pos: pos2(centre.x + dx * radius, centre.y + dy * radius),
                uv: pos2(uv_centre.0 + ux * uv_radius, uv_centre.1 + uy * uv_radius),
                color: Color32::WHITE,
            });
        }
        let segments = Self::DISC_SEGMENTS as u32;
        for i in 0..segments {
            mesh.add_triangle(0, 1 + i, 1 + (i + 1) % segments);
        }
        Shape::mesh(mesh)
    }
}

impl Default for DrawListShapes {
    fn default() -> Self {
        Self::new()
    }
}
