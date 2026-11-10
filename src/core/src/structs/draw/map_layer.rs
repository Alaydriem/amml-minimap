// which manager layer a pass paints: the corner map, or the world map and its panels
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapLayer {
    Hud,
    Overlay,
}
