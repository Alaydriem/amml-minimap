#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShutdownReport {
    // false when the worker did not finish within the time limit and was left running
    pub finished: bool,
}
