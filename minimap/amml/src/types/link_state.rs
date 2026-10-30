// where the link to the mod manager stands
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LinkState {
    // enabled; the next tick tries to reach the manager
    Waiting,
    Attached,
    // the manager is missing or refused the mod; the minimap stays off until the next enable
    Off,
}
