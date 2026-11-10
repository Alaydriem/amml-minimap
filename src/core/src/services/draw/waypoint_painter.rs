use std::fmt::Write;

use crate::config::Config;
use crate::services::draw::{MapRotation, WorldMapLayout};
use crate::structs::Rgba;
use crate::structs::draw::{DrawCommand, DrawList, MinimapFrame, TextAlign, WorldMapFrame};
use crate::structs::waypoint::{Waypoint, WaypointIcon, WaypointId};

// waypoint markers, labels and minimap edge arrows; only the frame's dimension is drawn
pub struct WaypointPainter {
    enabled: bool,
    show_on_minimap: bool,
    max_edge_arrows: usize,
    label_zoom: f32,
    widget_scale: f32,
    label_scale: f32,
    // (distance in blocks, index) of the waypoints outside the minimap that want an edge arrow
    edge: Vec<(f32, usize)>,
    // label boxes placed this frame, (x0, y0, x1, y1), so a new label can move below them
    placed: Vec<(f32, f32, f32, f32)>,
    // world map label boxes and their waypoints this frame, so a right click on a name finds it
    label_hits: Vec<((f32, f32, f32, f32), WaypointId)>,
    label: String,
}

impl WaypointPainter {
    pub const ICON: f32 = 6.0;
    pub const ARROW_LENGTH: f32 = 10.0;
    const ARROW_HALF_WIDTH: f32 = 6.0;
    const OUTLINE_WIDTH: f32 = 1.5;
    // a right click this many pixels past the icon still hits it
    const HIT_SLACK: f32 = 4.0;
    // an edge label's top is this far inside the arrow's base
    const EDGE_LABEL_INSET: f32 = 14.0;
    // icons and arrows are sized for this text size, and grow with the frame's text
    pub const BASE_FONT_SIZE: f32 = 18.0;
    // a rough glyph width in text sizes; core cannot measure the game's font
    const CHAR_WIDTH: f32 = 0.55;
    const LINE_HEIGHT: f32 = 1.25;
    // the dark band around a label, in text sizes
    const BAND_PAD: f32 = 0.2;
    const MAX_STACK: usize = 8;
    const OUTLINE: Rgba = Rgba { r: 0, g: 0, b: 0, a: 255 };
    const TEXT: Rgba = Rgba { r: 255, g: 255, b: 255, a: 255 };
    const BAND: Rgba = Rgba { r: 0, g: 0, b: 0, a: 170 };

    pub fn new(config: &Config) -> Self {
        Self {
            enabled: config.waypoints.enabled,
            show_on_minimap: config.waypoints.enabled && config.waypoints.show_on_minimap,
            max_edge_arrows: config.waypoints.max_edge_arrows.max(0) as usize,
            label_zoom: config.world_map.waypoint_label_zoom,
            widget_scale: config.mini_map.widget_scale,
            label_scale: config.waypoints.label_scale,
            edge: Vec::new(),
            placed: Vec::new(),
            label_hits: Vec::new(),
            label: String::new(),
        }
    }

    // disc is the minimap's (centre, radius) on screen
    pub fn paint_minimap(
        &mut self,
        frame: &MinimapFrame,
        disc: ((f32, f32), f32),
        waypoints: &[Waypoint],
        draw: &mut DrawList,
    ) {
        if !self.show_on_minimap {
            return;
        }
        self.placed.clear();
        self.edge.clear();
        let (centre, radius) = disc;
        let ws = self.unit(frame.font_size);
        let scale = radius / (frame.radius_chunks * 16) as f32;
        let turn = MapRotation::new(frame.angle, centre);
        let size = self.label_size(frame.font_size);
        let inside = radius - Self::ICON * ws;
        for (index, waypoint) in waypoints.iter().enumerate().filter(|(_, w)| w.dim == frame.dim) {
            let dx = waypoint.x as f32 + 0.5 - frame.camera.x;
            let dz = waypoint.z as f32 + 0.5 - frame.camera.z;
            let distance = dx.hypot(dz);
            if distance * scale <= inside {
                let at = turn.apply((centre.0 + dx * scale, centre.1 + dz * scale));
                Self::marker(draw, at, waypoint, ws);
                self.label.clear();
                self.label.push_str(&waypoint.name);
                self.place_label(draw, (at.0, at.1 - Self::ICON * ws - size * Self::LINE_HEIGHT), size);
            } else if waypoint.show_on_edge {
                self.edge.push((distance, index));
            }
        }
        self.edge.sort_unstable_by(|a, b| a.0.total_cmp(&b.0));
        for k in 0..self.edge.len().min(self.max_edge_arrows) {
            let (distance, index) = self.edge[k];
            let waypoint = &waypoints[index];
            let dx = waypoint.x as f32 + 0.5 - frame.camera.x;
            let dz = waypoint.z as f32 + 0.5 - frame.camera.z;
            let turned = turn.apply((centre.0 + dx / distance, centre.1 + dz / distance));
            let dir = (turned.0 - centre.0, turned.1 - centre.1);
            Self::edge_arrow(draw, centre, radius, dir, waypoint, ws);
            self.label.clear();
            self.label.push_str(&waypoint.name);
            if waypoint.show_distance {
                let _ = write!(self.label, " {} m", distance.round() as i64);
            }
            // the label is moved toward the centre by its own half width, so it stays inside the rim
            let reach = radius - (Self::ARROW_LENGTH + Self::EDGE_LABEL_INSET) * ws;
            let half = self.label_half(size);
            let anchor = (centre.0 + dir.0 * (reach - half), centre.1 + dir.1 * reach - size * 0.5);
            self.place_label(draw, anchor, size);
        }
    }

    // names are drawn from the configured zoom up
    pub fn paint_world_map(&mut self, frame: &WorldMapFrame<'_>, waypoints: &[Waypoint], draw: &mut DrawList) {
        if !self.enabled {
            return;
        }
        self.placed.clear();
        self.label_hits.clear();
        let (w, h) = frame.display;
        let ws = self.unit(frame.font_size);
        let turn = MapRotation::new(frame.angle, (w * 0.5, h * 0.5));
        let margin = Self::ICON * ws;
        let size = self.label_size(frame.font_size);
        let labels = frame.zoom >= self.label_zoom;
        for waypoint in waypoints.iter().filter(|w| w.dim == frame.dim) {
            let world = (f64::from(waypoint.x) + 0.5, f64::from(waypoint.z) + 0.5);
            let at = turn.apply(WorldMapLayout::to_screen(world, frame.centre, frame.zoom, frame.display));
            if at.0 < -margin || at.1 < -margin || at.0 > w + margin || at.1 > h + margin {
                continue;
            }
            Self::marker(draw, at, waypoint, ws);
            if labels {
                self.label.clear();
                self.label.push_str(&waypoint.name);
                let placed = self.place_label(draw, (at.0, at.1 - margin - size * Self::LINE_HEIGHT), size);
                self.label_hits.push((placed, waypoint.id));
            }
        }
    }

    // the marker under a world map screen point: the nearest one in the frame's dimension within reach,
    // else the name under it from the last paint, so a waypoint under another's marker is still found
    pub fn hit(&self, frame: &WorldMapFrame<'_>, waypoints: &[Waypoint], at: (f32, f32)) -> Option<WaypointId> {
        let (w, h) = frame.display;
        let turn = MapRotation::new(frame.angle, (w * 0.5, h * 0.5));
        let reach = (Self::ICON + Self::HIT_SLACK) * self.unit(frame.font_size);
        waypoints
            .iter()
            .filter(|wp| wp.dim == frame.dim)
            .map(|wp| {
                let world = (f64::from(wp.x) + 0.5, f64::from(wp.z) + 0.5);
                let p = turn.apply(WorldMapLayout::to_screen(world, frame.centre, frame.zoom, frame.display));
                ((p.0 - at.0).hypot(p.1 - at.1), wp.id)
            })
            .filter(|(d, _)| *d <= reach)
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(_, id)| id)
            .or_else(|| {
                self.label_hits
                    .iter()
                    .find(|((x0, y0, x1, y1), _)| at.0 >= *x0 && at.0 < *x1 && at.1 >= *y0 && at.1 < *y1)
                    .map(|(_, id)| *id)
            })
    }

    // the pixel size of one unit of icon and arrow geometry
    fn unit(&self, font_size: f32) -> f32 {
        self.widget_scale * font_size / Self::BASE_FONT_SIZE
    }

    fn label_size(&self, font_size: f32) -> f32 {
        font_size * self.widget_scale * self.label_scale
    }

    fn label_half(&self, size: f32) -> f32 {
        self.label.chars().count() as f32 * size * Self::CHAR_WIDTH * 0.5
    }

    // a diamond, or a skull for the death point, over a black outline
    fn marker(draw: &mut DrawList, at: (f32, f32), waypoint: &Waypoint, ws: f32) {
        let [r, g, b, a] = waypoint.colour;
        let colour = Rgba::new(r, g, b, a).packed();
        let outline = Self::OUTLINE.packed();
        let size = Self::ICON * ws;
        match waypoint.icon {
            WaypointIcon::Diamond => {
                Self::diamond(draw, at, size + Self::OUTLINE_WIDTH * ws, outline);
                Self::diamond(draw, at, size, colour);
            }
            WaypointIcon::Skull => {
                let eye = size * 0.25;
                draw.push(DrawCommand::CircleFilled {
                    centre: at,
                    radius: size + Self::OUTLINE_WIDTH * ws,
                    colour: outline,
                });
                draw.push(DrawCommand::CircleFilled { centre: at, radius: size, colour });
                for side in [-1.0, 1.0] {
                    let centre = (at.0 + side * size * 0.38, at.1 - size * 0.1);
                    draw.push(DrawCommand::CircleFilled { centre, radius: eye, colour: outline });
                }
            }
        }
    }

    fn diamond(draw: &mut DrawList, at: (f32, f32), size: f32, colour: u32) {
        let (top, right, bottom, left) =
            ((at.0, at.1 - size), (at.0 + size, at.1), (at.0, at.1 + size), (at.0 - size, at.1));
        draw.push(DrawCommand::TriangleFilled { a: top, b: right, c: bottom, colour });
        draw.push(DrawCommand::TriangleFilled { a: top, b: bottom, c: left, colour });
    }

    // dir is a unit screen vector from the centre toward the waypoint; the tip touches the rim
    fn edge_arrow(draw: &mut DrawList, centre: (f32, f32), radius: f32, dir: (f32, f32), waypoint: &Waypoint, ws: f32) {
        let [r, g, b, a] = waypoint.colour;
        let along =
            |d: f32, across: f32| (centre.0 + dir.0 * d - dir.1 * across, centre.1 + dir.1 * d + dir.0 * across);
        let base = radius - Self::ARROW_LENGTH * ws;
        let half = Self::ARROW_HALF_WIDTH * ws;
        let pad = Self::OUTLINE_WIDTH * ws;
        draw.push(DrawCommand::TriangleFilled {
            a: along(radius + pad, 0.0),
            b: along(base - pad, -half - pad),
            c: along(base - pad, half + pad),
            colour: Self::OUTLINE.packed(),
        });
        draw.push(DrawCommand::TriangleFilled {
            a: along(radius, 0.0),
            b: along(base, -half),
            c: along(base, half),
            colour: Rgba::new(r, g, b, a).packed(),
        });
    }

    // draws self.label centred on `anchor` on a dark band, moved up a line at a time past labels
    // already placed, so a stacked name never covers the marker below it
    fn place_label(&mut self, draw: &mut DrawList, anchor: (f32, f32), size: f32) -> (f32, f32, f32, f32) {
        let half = self.label_half(size);
        let mut top = anchor.1;
        for _ in 0..Self::MAX_STACK {
            let overlaps = self
                .placed
                .iter()
                .any(|&(x0, y0, x1, y1)| anchor.0 - half < x1 && anchor.0 + half > x0 && top < y1 && top + size > y0);
            if !overlaps {
                break;
            }
            top -= size * Self::LINE_HEIGHT;
        }
        self.placed.push((anchor.0 - half, top, anchor.0 + half, top + size));
        let pad = size * Self::BAND_PAD;
        draw.push(DrawCommand::RectFilled {
            p0: (anchor.0 - half - pad, top - pad * 0.5),
            p1: (anchor.0 + half + pad, top + size + pad * 0.5),
            colour: Self::BAND.packed(),
        });
        draw.text((anchor.0, top), Self::TEXT.packed(), size, TextAlign::Centre, &self.label);
        (anchor.0 - half - pad, top - pad * 0.5, anchor.0 + half + pad, top + size + pad * 0.5)
    }
}
