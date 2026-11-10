use crate::config::MiniMapConfig;
use crate::services::draw::{MapRotation, OverlayGate, WorldMapLayout};
use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, ExploredChunks, WorldMapFrame};
use crate::structs::pos::ChunkPos;

// chunk borders over the world map's explored chunks, and the chunk under the cursor tinted and
// outlined in the UI's accent
pub struct ChunkGridPainter {
    line: u32,
    thickness: f32,
}

impl ChunkGridPainter {
    // the accent #8B7CFF as IM_COL32: opaque for the outline, at 16 % for the fill
    pub const HOVER_OUTLINE: u32 = 0xFFFF_7C8B;
    pub const HOVER_FILL: u32 = 0x29FF_7C8B;
    const HOVER_WIDTH: f32 = 2.0;

    pub fn new(config: &MiniMapConfig) -> Self {
        Self { line: Self::colour(config.chunk_line_color), thickness: config.line_thickness }
    }

    // the borders of explored chunks only: a border is drawn where either chunk beside it is
    // explored, as one line per run of such borders. The hovered chunk is tinted when explored
    pub fn paint(
        &self,
        frame: &WorldMapFrame<'_>,
        hovered: Option<(i32, i32)>,
        explored: &ExploredChunks,
        draw: &mut DrawList,
    ) {
        if !OverlayGate::open(frame.zoom) {
            return;
        }
        let (cx0, cx1, cz0, cz1) = WorldMapLayout::visible_chunks(frame.centre, frame.zoom, frame.display, frame.angle);
        let side = ChunkPos::BLOCKS;
        for cx in cx0..=cx1 + 1 {
            let edge = |cz| explored.contains(cx - 1, cz) || explored.contains(cx, cz);
            Self::runs(cz0, cz1, edge, |from, to| {
                Self::line(
                    frame,
                    (cx * side, from * side),
                    (cx * side, (to + 1) * side),
                    self.line,
                    self.thickness,
                    draw,
                );
            });
        }
        for cz in cz0..=cz1 + 1 {
            let edge = |cx| explored.contains(cx, cz - 1) || explored.contains(cx, cz);
            Self::runs(cx0, cx1, edge, |from, to| {
                Self::line(
                    frame,
                    (from * side, cz * side),
                    ((to + 1) * side, cz * side),
                    self.line,
                    self.thickness,
                    draw,
                );
            });
        }
        let Some((cx, cz)) = hovered.filter(|&(cx, cz)| explored.contains(cx, cz)) else { return };
        let (x0, z0, x1, z1) = (cx * side, cz * side, (cx + 1) * side, (cz + 1) * side);
        let corners = [(x0, z0), (x1, z0), (x1, z1), (x0, z1)].map(|corner| Self::screen(frame, corner));
        for (a, b, c) in [(0, 1, 2), (0, 2, 3)] {
            draw.push(DrawCommand::TriangleFilled {
                a: corners[a],
                b: corners[b],
                c: corners[c],
                colour: Self::HOVER_FILL,
            });
        }
        for i in 0..corners.len() {
            let (a, b) = (corners[i], corners[(i + 1) % corners.len()]);
            draw.push(DrawCommand::Line { a, b, colour: Self::HOVER_OUTLINE, thickness: Self::HOVER_WIDTH });
        }
    }

    // calls `emit(first, last)` for each run of `from..=to` where `on` holds
    fn runs(from: i32, to: i32, on: impl Fn(i32) -> bool, mut emit: impl FnMut(i32, i32)) {
        let mut start: Option<i32> = None;
        for i in from..=to + 1 {
            match (i <= to && on(i), start) {
                (true, None) => start = Some(i),
                (false, Some(first)) => {
                    emit(first, i - 1);
                    start = None;
                }
                _ => {}
            }
        }
    }

    fn line(frame: &WorldMapFrame<'_>, a: (i32, i32), b: (i32, i32), colour: u32, thickness: f32, draw: &mut DrawList) {
        draw.push(DrawCommand::Line { a: Self::screen(frame, a), b: Self::screen(frame, b), colour, thickness });
    }

    // a block corner on the turned screen
    pub(crate) fn screen(frame: &WorldMapFrame<'_>, block: (i32, i32)) -> (f32, f32) {
        let flat = WorldMapLayout::to_screen(
            (f64::from(block.0), f64::from(block.1)),
            frame.centre,
            frame.zoom,
            frame.display,
        );
        if frame.angle == 0.0 {
            return flat;
        }
        MapRotation::new(frame.angle, (frame.display.0 * 0.5, frame.display.1 * 0.5)).apply(flat)
    }

    // config colours are 0xRRGGBBAA
    fn colour(rrggbbaa: u32) -> u32 {
        let [r, g, b, a] = rrggbbaa.to_be_bytes();
        Rgba::new(r, g, b, a).packed()
    }
}
