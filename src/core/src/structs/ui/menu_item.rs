#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    AddWaypoint,
    CopyCoordinates,
    ToggleWaypoints,
    EditWaypoint,
    RemoveWaypoint,
    ExportView,
    ExportArea,
}

impl MenuItem {
    // the toggle names what it will do
    pub fn label(&self, waypoints_shown: bool) -> &'static str {
        match self {
            Self::AddWaypoint => "Add waypoint here",
            Self::CopyCoordinates => "Copy coordinates",
            Self::ToggleWaypoints if waypoints_shown => "Hide waypoints",
            Self::ToggleWaypoints => "Show waypoints",
            Self::EditWaypoint => "Edit",
            Self::RemoveWaypoint => "Remove",
            Self::ExportView => "Export this view",
            Self::ExportArea => "Export explored area",
        }
    }
}
