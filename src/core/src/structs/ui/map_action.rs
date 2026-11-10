use crate::structs::ui::MenuItem;

// what a world map panel asks for; the runtime applies it to the map state after the UI frame
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MapAction {
    ToggleGrid,
    ToggleSlime,
    ToggleList,
    // hold the map on this world point, the middle of a waypoint's block
    CentreOn { x: f64, z: f64 },
    // a row of the open menu, for its target
    Choose(MenuItem),
    CloseMenu,
    DialogColour([u8; 4]),
    DialogEdge,
    DialogDistance,
    // saves with the typed name the runtime passes in
    SaveDialog,
    CancelDialog,
    // Escape on the world map with no menu or dialog open
    CloseWorldMap,
    // Ctrl+Z while the last removal can be undone
    UndoRemove,
}
