// how much bigger text is drawn on a tall screen: sizes in config are for a 1080-pixel-high screen,
// and a shorter screen keeps them as they are
pub struct TextScale;

impl TextScale {
    pub const REFERENCE_HEIGHT: f32 = 1080.0;

    pub fn for_display(height: f32) -> f32 {
        (height / Self::REFERENCE_HEIGHT).max(1.0)
    }
}
