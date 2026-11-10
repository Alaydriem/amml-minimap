use std::path::PathBuf;

use minimap_core::services::export::AreaLayout;
use minimap_core::structs::pos::RegionPos;

// one export for the export thread; `epoch` is the world visit it belongs to
#[derive(Debug)]
pub enum ExportJob {
    // `regions` are drawn, from the tile store or the region files under `cache`; the rest is clear
    Area { path: PathBuf, layout: AreaLayout, regions: Vec<RegionPos>, cache: Option<PathBuf>, dim: i32, epoch: u64 },
}
