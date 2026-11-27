use crate::types::BlockKind;

// the full name of one block type and its kind, kept by `BlockNames`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockName {
    pub name: Box<str>,
    pub kind: BlockKind,
}

impl BlockName {
    pub fn new(name: &str) -> Self {
        Self { name: name.into(), kind: BlockKind::of(name) }
    }
}
