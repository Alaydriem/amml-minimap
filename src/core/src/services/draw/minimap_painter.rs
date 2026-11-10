use crate::config::MiniMapConfig;
use crate::services::draw::{CircleClip, CoordinateText, MapRotation};
use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, MinimapFrame, TextAlign};

// the minimap widget: disc, terrain texture, chunk grid, current chunk, arrow, coordinates, border
pub struct MinimapPainter {
    config: MiniMapConfig,
}

impl MinimapPainter {
    const LINE_CLIP_INSET: f32 = 1.0;
    // the second coordinate line sits this many text sizes below the first
    const LINE_SPACING: f32 = 1.15;

    pub fn new(config: &MiniMapConfig) -> Self {
        Self { config: config.clone() }
    }

    // centre and radius of the disc in screen pixels
    pub fn disc(&self, display_width: f32) -> ((f32, f32), f32) {
        let scale = self.config.widget_scale;
        let radius = display_width * self.config.size_ratio * scale * 0.5;
        let centre =
            (display_width - self.config.margin_right * scale - radius, self.config.margin_top * scale + radius);
        (centre, radius)
    }

    pub fn paint(&self, frame: &MinimapFrame, draw: &mut DrawList) {
        let c = &self.config;
        let ws = c.widget_scale;
        let (centre, radius) = self.disc(frame.display_width);
        let blocks = (frame.radius_chunks * 16) as f32;
        let scale = radius / blocks;
        let turn = MapRotation::new(frame.angle, centre);
        let to_screen = |wx: f32, wz: f32| {
            turn.apply((centre.0 + (wx - frame.camera.x) * scale, centre.1 + (wz - frame.camera.z) * scale))
        };

        draw.push(DrawCommand::CircleFilled { centre, radius, colour: Self::colour(c.background_color) });
        if let Some(tex) = frame.texture {
            let side = frame.image_side as f32;
            let u = |wx: f32| (wx - frame.image_origin.0 as f32) / side;
            let v = |wz: f32| (wz - frame.image_origin.1 as f32) / side;
            draw.push(DrawCommand::ImageDisc {
                tex,
                centre,
                radius,
                uv_centre: (u(frame.camera.x), v(frame.camera.z)),
                uv_radius: blocks / side,
                angle: frame.angle,
            });
        }

        let clip = radius - Self::LINE_CLIP_INSET;
        let thickness = c.line_thickness * ws;
        let chunk_x = (frame.camera.x / 16.0).floor() as i32;
        let chunk_z = (frame.camera.z / 16.0).floor() as i32;
        let r = frame.radius_chunks;
        let (x0, x1) = (((chunk_x - r) * 16) as f32, ((chunk_x + r + 1) * 16) as f32);
        let (z0, z1) = (((chunk_z - r) * 16) as f32, ((chunk_z + r + 1) * 16) as f32);
        let line_colour = Self::colour(c.chunk_line_color);
        for z in (chunk_z - r)..=(chunk_z + r + 1) {
            let wz = (z * 16) as f32;
            Self::clipped_line(draw, (to_screen(x0, wz), to_screen(x1, wz)), centre, clip, (line_colour, thickness));
        }
        for x in (chunk_x - r)..=(chunk_x + r + 1) {
            let wx = (x * 16) as f32;
            Self::clipped_line(draw, (to_screen(wx, z0), to_screen(wx, z1)), centre, clip, (line_colour, thickness));
        }
        let (cx0, cz0) = ((chunk_x * 16) as f32, (chunk_z * 16) as f32);
        let corners = [
            to_screen(cx0, cz0),
            to_screen(cx0 + 16.0, cz0),
            to_screen(cx0 + 16.0, cz0 + 16.0),
            to_screen(cx0, cz0 + 16.0),
        ];
        let current = Self::colour(c.current_chunk_color);
        for i in 0..4 {
            Self::clipped_line(draw, (corners[i], corners[(i + 1) % 4]), centre, clip, (current, thickness));
        }

        let (sin, cos) = (frame.camera.render_yaw_rad() + frame.angle).sin_cos();
        let rotate = |x: f32, z: f32| (centre.0 + (x * cos - z * sin) * ws, centre.1 + (x * sin + z * cos) * ws);
        draw.push(DrawCommand::TriangleFilled {
            a: rotate(0.0, -10.0),
            b: rotate(-7.0, 10.0),
            c: rotate(7.0, 10.0),
            colour: Self::colour(c.player_arrow_outline_color),
        });
        draw.push(DrawCommand::TriangleFilled {
            a: rotate(0.0, -8.0),
            b: rotate(-5.0, 8.0),
            c: rotate(5.0, 8.0),
            colour: Self::colour(c.player_arrow_fill_color),
        });

        if frame.show_coordinates {
            let (x, y, z) = frame.player_block;
            let text = format!("{x}, {y}, {z}");
            let anchor = (centre.0, centre.1 + radius + c.coord_text_margin * ws);
            let size = frame.font_size * ws;
            draw.text(
                (anchor.0 + 1.0, anchor.1 + 1.0),
                Self::colour(c.coord_shadow_color),
                size,
                TextAlign::Centre,
                &text,
            );
            draw.text(anchor, Self::colour(c.coord_text_color), size, TextAlign::Centre, &text);
            if c.show_nether_coordinates
                && let Some(linked) = CoordinateText::linked(frame.dim, x, z)
            {
                let below = (anchor.0, anchor.1 + size * Self::LINE_SPACING);
                draw.text(
                    (below.0 + 1.0, below.1 + 1.0),
                    Self::colour(c.coord_shadow_color),
                    size,
                    TextAlign::Centre,
                    &linked,
                );
                draw.text(below, Self::colour(c.coord_text_color), size, TextAlign::Centre, &linked);
            }
        }

        draw.push(DrawCommand::Circle {
            centre,
            radius,
            colour: Self::colour(c.border_color),
            thickness: c.border_thickness * ws,
        });
    }

    // stroke is (colour, thickness)
    fn clipped_line(
        draw: &mut DrawList,
        segment: ((f32, f32), (f32, f32)),
        centre: (f32, f32),
        radius: f32,
        stroke: (u32, f32),
    ) {
        let (colour, thickness) = stroke;
        if let Some((a, b)) = CircleClip::clip(segment.0, segment.1, centre, radius) {
            draw.push(DrawCommand::Line { a, b, colour, thickness });
        }
    }

    // config colours are 0xRRGGBBAA; draw colours are packed like IM_COL32
    fn colour(rrggbbaa: u32) -> u32 {
        let [r, g, b, a] = rrggbbaa.to_be_bytes();
        Rgba::new(r, g, b, a).packed()
    }
}
