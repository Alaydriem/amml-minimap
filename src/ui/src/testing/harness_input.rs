use crate::testing::HarnessEvent;

// what one test pass is given, the way the manager builds a layer's input: the screen, the scale, the
// pointer and the queued keys, text and wheel
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HarnessInput<'a> {
    pub display: (f32, f32),
    // physical pixels per UI point
    pub pixels_per_point: f32,
    pub time_s: f64,
    // client-area cursor position in pixels
    pub cursor: (f32, f32),
    pub left_down: bool,
    pub right_down: bool,
    pub foreground: bool,
    pub events: &'a [HarnessEvent],
}

impl<'a> HarnessInput<'a> {
    pub fn new(display: (f32, f32), cursor: (f32, f32), left_down: bool, events: &'a [HarnessEvent]) -> Self {
        Self {
            display,
            pixels_per_point: 1.0,
            time_s: 0.0,
            cursor,
            left_down,
            right_down: false,
            foreground: true,
            events,
        }
    }
}
