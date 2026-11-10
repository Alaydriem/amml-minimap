// the map layer shown and scanned: a dimension's surface, or one cave band of it.
// `id` stands in for the dimension in chunk and region positions and in the disk folders
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MapLayer {
    pub id: i32,
    // the real dimension
    pub dim: i32,
    pub band: Option<i32>,
    // the probe's starting height
    pub camera_height: i32,
    pub scan_radius_chunks: i32,
}
