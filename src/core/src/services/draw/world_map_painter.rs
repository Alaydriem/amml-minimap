use crate::services::draw::{CoordinateText, MapRotation, WorldMapLayout};
use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, SamplerKind, TextAlign, WorldMapFrame};
use crate::structs::pos::RegionPos;

// the full-screen world map: background, region tiles or placeholders, player arrow, bottom text
pub struct WorldMapPainter;

impl WorldMapPainter {
    const BACKGROUND: Rgba = Rgba { r: 16, g: 16, b: 20, a: 255 };
    const PLACEHOLDER: Rgba = Rgba { r: 28, g: 28, b: 34, a: 255 };
    const ARROW_OUTLINE: Rgba = Rgba { r: 0, g: 0, b: 0, a: 255 };
    const ARROW_FILL: Rgba = Rgba { r: 220, g: 50, b: 50, a: 255 };
    const TEXT: Rgba = Rgba { r: 255, g: 255, b: 255, a: 255 };
    const TEXT_SHADOW: Rgba = Rgba { r: 0, g: 0, b: 0, a: 200 };
    const TEXT_BOTTOM_MARGIN: f32 = 10.0;

    pub fn paint(frame: &WorldMapFrame<'_>, draw: &mut DrawList) {
        let (w, h) = frame.display;
        draw.push(DrawCommand::RectFilled { p0: (0.0, 0.0), p1: (w, h), colour: Self::BACKGROUND.packed() });
        let point = frame.zoom >= 1.0;
        draw.push(DrawCommand::Sampler(if point { SamplerKind::Point } else { SamplerKind::Linear }));
        let side = f64::from(RegionPos::BLOCKS);
        let turn = MapRotation::new(frame.angle, (w * 0.5, h * 0.5));
        for &(region, tex) in frame.regions {
            let world0 = (f64::from(region.x) * side, f64::from(region.z) * side);
            let world1 = (world0.0 + side, world0.1 + side);
            let p0 = WorldMapLayout::to_screen(world0, frame.centre, frame.zoom, frame.display);
            let p1 = WorldMapLayout::to_screen(world1, frame.centre, frame.zoom, frame.display);
            if turn.is_turned() {
                Self::turned_region(draw, &turn, (p0, p1), tex);
                continue;
            }
            let (p0, p1) = ((p0.0.floor(), p0.1.floor()), (p1.0.floor(), p1.1.floor()));
            match tex {
                Some(tex) => draw.push(DrawCommand::Image { tex, p0, p1, uv0: (0.0, 0.0), uv1: (1.0, 1.0) }),
                None => draw.push(DrawCommand::RectFilled { p0, p1, colour: Self::PLACEHOLDER.packed() }),
            }
        }
        if point {
            draw.push(DrawCommand::Sampler(SamplerKind::Linear));
        }

        let player = (f64::from(frame.camera.x), f64::from(frame.camera.z));
        let pp = turn.apply(WorldMapLayout::to_screen(player, frame.centre, frame.zoom, frame.display));
        let (sin, cos) = (frame.camera.render_yaw_rad() + frame.angle).sin_cos();
        let rotate = |x: f32, z: f32| (pp.0 + (x * cos - z * sin), pp.1 + (x * sin + z * cos));
        draw.push(DrawCommand::TriangleFilled {
            a: rotate(0.0, -12.0),
            b: rotate(-7.0, 12.0),
            c: rotate(7.0, 12.0),
            colour: Self::ARROW_OUTLINE.packed(),
        });
        draw.push(DrawCommand::TriangleFilled {
            a: rotate(0.0, -9.0),
            b: rotate(-5.0, 12.0),
            c: rotate(5.0, 12.0),
            colour: Self::ARROW_FILL.packed(),
        });

        if frame.show_coordinates {
            let (x, _, z) = frame.player_block;
            let linked = if frame.show_nether_coordinates { CoordinateText::linked(frame.dim, x, z) } else { None };
            let text = match linked {
                Some(linked) => format!("X: {x}  Z: {z}   {linked}   Zoom: {:.2}x", frame.zoom),
                None => format!("X: {x}  Z: {z}   Zoom: {:.2}x", frame.zoom),
            };
            let anchor = (w * 0.5, h - Self::TEXT_BOTTOM_MARGIN);
            draw.text(
                (anchor.0 + 1.0, anchor.1 + 1.0),
                Self::TEXT_SHADOW.packed(),
                frame.font_size,
                TextAlign::CentreBottom,
                &text,
            );
            draw.text(anchor, Self::TEXT.packed(), frame.font_size, TextAlign::CentreBottom, &text);
        }
    }

    // a region drawn turned around the screen centre: an image quad, or two placeholder triangles
    fn turned_region(draw: &mut DrawList, turn: &MapRotation, rect: ((f32, f32), (f32, f32)), tex: Option<u32>) {
        let (p0, p1) = rect;
        let middle = turn.apply(((p0.0 + p1.0) * 0.5, (p0.1 + p1.1) * 0.5));
        let half = ((p1.0 - p0.0) * 0.5, (p1.1 - p0.1) * 0.5);
        match tex {
            Some(tex) => draw.push(DrawCommand::ImageRotated {
                tex,
                centre: middle,
                half,
                uv0: (0.0, 0.0),
                uv1: (1.0, 1.0),
                angle: turn.angle(),
            }),
            None => {
                let corners = [turn.apply(p0), turn.apply((p1.0, p0.1)), turn.apply(p1), turn.apply((p0.0, p1.1))];
                let colour = Self::PLACEHOLDER.packed();
                draw.push(DrawCommand::TriangleFilled { a: corners[0], b: corners[1], c: corners[2], colour });
                draw.push(DrawCommand::TriangleFilled { a: corners[0], b: corners[2], c: corners[3], colour });
            }
        }
    }
}
