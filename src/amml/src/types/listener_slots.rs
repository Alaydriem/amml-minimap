// the resolved vtable slots the block listener uses: three on BlockSource, two on BlockSourceListener
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ListenerSlots {
    pub add_listener: u64,
    pub remove_listener: u64,
    pub dimension_id: u64,
    pub source_destroyed: u64,
    pub block_changed: u64,
}
