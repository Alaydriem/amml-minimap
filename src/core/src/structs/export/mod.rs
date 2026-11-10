pub mod view_regions;

pub use view_regions::ViewRegions;

// what an export writes: the regions on screen, or every explored region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportKind {
    View,
    Area,
}

impl ExportKind {
    // the last part of the file name
    pub fn suffix(&self) -> &'static str {
        match self {
            Self::View => "view",
            Self::Area => "area",
        }
    }
}
