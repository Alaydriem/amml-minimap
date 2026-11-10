use serde::Deserialize;

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(default, rename_all = "snake_case")]
pub struct ExportConfig {
    // the explored-area image is at most this many pixels wide and tall
    pub max_pixels: i32,
    // the folder export files go to; empty means the user's Pictures/Screenshots folder
    pub folder: String,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self { max_pixels: 16384, folder: String::new() }
    }
}
