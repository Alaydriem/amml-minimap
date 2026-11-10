use crate::services::draw::MapRotation;
use crate::structs::pos::{ChunkPos, RegionPos};

// which regions cover the screen at a given centre and zoom
pub struct WorldMapLayout;

impl WorldMapLayout {
    // inclusive (rx0, rx1, rz0, rz1)
    pub fn visible(centre: (f64, f64), zoom: f32, display: (f32, f32)) -> (i32, i32, i32, i32) {
        Self::span(centre, zoom, display, f64::from(RegionPos::BLOCKS))
    }

    // inclusive cells of `side` blocks that cover the screen
    fn span(centre: (f64, f64), zoom: f32, display: (f32, f32), side: f64) -> (i32, i32, i32, i32) {
        let half_w = f64::from(display.0) * 0.5 / f64::from(zoom);
        let half_h = f64::from(display.1) * 0.5 / f64::from(zoom);
        (
            ((centre.0 - half_w) / side).floor() as i32,
            ((centre.0 + half_w) / side).floor() as i32,
            ((centre.1 - half_h) / side).floor() as i32,
            ((centre.1 + half_h) / side).floor() as i32,
        )
    }

    // the regions under a screen turned by any angle: the screen's diagonal square
    pub fn visible_turned(centre: (f64, f64), zoom: f32, display: (f32, f32)) -> (i32, i32, i32, i32) {
        let diagonal = display.0.hypot(display.1);
        Self::visible(centre, zoom, (diagonal, diagonal))
    }

    // inclusive (cx0, cx1, cz0, cz1) of the chunks on screen; a turned map uses the diagonal square
    pub fn visible_chunks(centre: (f64, f64), zoom: f32, display: (f32, f32), angle: f32) -> (i32, i32, i32, i32) {
        let display = if angle == 0.0 {
            display
        } else {
            let diagonal = display.0.hypot(display.1);
            (diagonal, diagonal)
        };
        Self::span(centre, zoom, display, f64::from(ChunkPos::BLOCKS))
    }

    // the block under a screen point of a map turned by `angle` radians around the screen centre
    pub fn block_at(screen: (f32, f32), centre: (f64, f64), zoom: f32, display: (f32, f32), angle: f32) -> (i32, i32) {
        let (w, h) = display;
        let flat = MapRotation::new(-angle, (w * 0.5, h * 0.5)).apply(screen);
        let (x, z) = Self::to_world(flat, centre, zoom, display);
        (x.floor() as i32, z.floor() as i32)
    }

    // the world block position under a screen point; the inverse of `to_screen`
    pub fn to_world(screen: (f32, f32), centre: (f64, f64), zoom: f32, display: (f32, f32)) -> (f64, f64) {
        let zoom = f64::from(zoom);
        (
            (f64::from(screen.0) - f64::from(display.0) * 0.5) / zoom + centre.0,
            (f64::from(screen.1) - f64::from(display.1) * 0.5) / zoom + centre.1,
        )
    }

    pub fn to_screen(world: (f64, f64), centre: (f64, f64), zoom: f32, display: (f32, f32)) -> (f32, f32) {
        let zoom = f64::from(zoom);
        (
            ((world.0 - centre.0) * zoom + f64::from(display.0) * 0.5) as f32,
            ((world.1 - centre.1) * zoom + f64::from(display.1) * 0.5) as f32,
        )
    }
}
