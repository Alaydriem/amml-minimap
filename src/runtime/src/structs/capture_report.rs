// what a capture writer did; dropped events never block the game, so a capture can have gaps
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CaptureReport {
    pub written: u64,
    pub dropped: u64,
    pub finished: bool,
}
