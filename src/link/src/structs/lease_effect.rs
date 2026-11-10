// what the runtime does because a lease changed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseEffect {
    CloseWorldMap,
    // the manager disabled the mod: stop scanning and drawing
    Pause,
    Resume,
}
