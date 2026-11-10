#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DirtyEpochs {
    pub full: u32,
    pub soft: u32,
}
