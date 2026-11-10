use amml_i18n::message::{Domain, Message};

/// The words the minimap's world map views paint. Text that minimap-core and the runtime build (menu items,
/// dimension and biome names, toasts) stays English for now and is shown as it comes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MinimapMessage {
    WorldMap,
    Overlays,
    ChunkGrid,
    SlimeChunks,
    ZoomIn,
    UnderCursor,
    Waypoints,
    WaypointsInDimension,
    NoWaypointsHere,
    SearchWaypoints,
    ListFooter,
    Here,
    BlocksAway,
    Hint,
    Block,
    Chunk,
    Biome,
    BlockAt,
    Waypoint,
    AxisX,
    AxisY,
    AxisZ,
    NewWaypoint,
    EditWaypoint,
    Name,
    Colour,
    OnTheMinimap,
    Position,
    ArrowAtEdge,
    ShowDistance,
    Save,
}

impl Message for MinimapMessage {
    const DOMAIN: Domain = Domain::new("minimap");
    const ALL: &'static [Self] = &[
        Self::WorldMap,
        Self::Overlays,
        Self::ChunkGrid,
        Self::SlimeChunks,
        Self::ZoomIn,
        Self::UnderCursor,
        Self::Waypoints,
        Self::WaypointsInDimension,
        Self::NoWaypointsHere,
        Self::SearchWaypoints,
        Self::ListFooter,
        Self::Here,
        Self::BlocksAway,
        Self::Hint,
        Self::Block,
        Self::Chunk,
        Self::Biome,
        Self::BlockAt,
        Self::Waypoint,
        Self::AxisX,
        Self::AxisY,
        Self::AxisZ,
        Self::NewWaypoint,
        Self::EditWaypoint,
        Self::Name,
        Self::Colour,
        Self::OnTheMinimap,
        Self::Position,
        Self::ArrowAtEdge,
        Self::ShowDistance,
        Self::Save,
    ];

    fn index(self) -> usize {
        self as usize
    }

    fn msgid(self) -> &'static str {
        match self {
            Self::WorldMap => "World map",
            Self::Overlays => "OVERLAYS",
            Self::ChunkGrid => "Chunk grid",
            Self::SlimeChunks => "Slime chunks",
            Self::ZoomIn => "zoom in",
            Self::UnderCursor => "UNDER CURSOR",
            Self::Waypoints => "Waypoints",
            Self::WaypointsInDimension => "{count} in this dimension",
            Self::NoWaypointsHere => "No waypoints here",
            Self::SearchWaypoints => "Search waypoints",
            Self::ListFooter => "Click a row to centre the map  \u{b7}  wheel scrolls",
            Self::Here => "Here",
            Self::BlocksAway => "{count} m away",
            Self::Hint => "{key} close  \u{b7}  right-click for actions  \u{b7}  drag to pan",
            Self::Block => "Block",
            Self::Chunk => "Chunk",
            Self::Biome => "Biome",
            Self::BlockAt => "BLOCK {x}, {z}",
            Self::Waypoint => "WAYPOINT",
            Self::AxisX => "X",
            Self::AxisY => "Y",
            Self::AxisZ => "Z",
            Self::NewWaypoint => "New waypoint",
            Self::EditWaypoint => "Edit waypoint",
            Self::Name => "NAME",
            Self::Colour => "COLOUR",
            Self::OnTheMinimap => "ON THE MINIMAP",
            Self::Position => "POSITION",
            Self::ArrowAtEdge => "Arrow at the edge when far away",
            Self::ShowDistance => "Show distance on the arrow",
            Self::Save => "Save",
        }
    }

    fn context(self) -> Option<&'static str> {
        None
    }

    fn plural(self) -> Option<&'static str> {
        match self {
            Self::WaypointsInDimension => Some("{count} in this dimension"),
            Self::BlocksAway => Some("{count} m away"),
            _ => None,
        }
    }

    fn note(self) -> Option<&'static str> {
        match self {
            Self::Overlays | Self::UnderCursor | Self::Name | Self::Colour | Self::OnTheMinimap | Self::Position => {
                Some("A section caption, painted in spaced capitals")
            }
            Self::ZoomIn => Some("Beside a switch that works only when the map is zoomed in"),
            Self::WaypointsInDimension => Some("How many waypoints the list shows, in the dimension on the map"),
            Self::Here => Some("The distance to a waypoint when the player stands at it"),
            Self::BlocksAway => Some("The distance to a waypoint; m is metres, one block each"),
            Self::Hint => Some("{key} is the key chord that opens and closes the world map"),
            Self::BlockAt => Some("The right-click menu title on open ground, in capitals: the block's x and z"),
            Self::Waypoint => Some("The right-click menu title on a waypoint that has no name, in capitals"),
            Self::AxisX | Self::AxisY | Self::AxisZ => Some("A world axis before its coordinate"),
            _ => None,
        }
    }
}
