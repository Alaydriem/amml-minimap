#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeaseState {
    Released,
    Held,
    // another mod's exclusive claim holds it for now; the manager sends Resumed when it ends
    Suspended,
    // the manager said no; asked again only after CapabilityAvailable, or on the next world map open
    Refused,
}

impl LeaseState {
    // the manager counts the lease as this mod's, so it is released when no longer wanted
    pub fn is_taken(self) -> bool {
        matches!(self, Self::Held | Self::Suspended)
    }
}
