use crate::structs::ui::{MenuItem, MenuTarget};

// an open right-click menu; `at` is the cursor that opened it
#[derive(Debug, Clone, PartialEq)]
pub struct MapMenu {
    pub at: (f32, f32),
    pub target: MenuTarget,
    pub items: Vec<MenuItem>,
}

impl MapMenu {
    pub fn new(at: (f32, f32), target: MenuTarget) -> Self {
        let items = match target {
            MenuTarget::Ground { .. } => vec![
                MenuItem::AddWaypoint,
                MenuItem::CopyCoordinates,
                MenuItem::ToggleWaypoints,
                MenuItem::ExportView,
                MenuItem::ExportArea,
            ],
            MenuTarget::Marker(_) => vec![MenuItem::EditWaypoint, MenuItem::RemoveWaypoint, MenuItem::CopyCoordinates],
        };
        Self { at, target, items }
    }
}
