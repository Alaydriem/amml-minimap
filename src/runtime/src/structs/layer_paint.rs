use minimap_core::structs::draw::DrawList;

use crate::structs::UploadStage;

// what the last pass of one layer drew and uploaded; both keep their capacity between passes
#[derive(Debug, Default)]
pub struct LayerPaint {
    pub draw: DrawList,
    pub stage: UploadStage,
}

impl LayerPaint {
    pub fn new() -> Self {
        Self { draw: DrawList::new(), stage: UploadStage::new() }
    }

    pub fn clear(&mut self) {
        self.draw.clear();
        self.stage.clear();
    }
}
