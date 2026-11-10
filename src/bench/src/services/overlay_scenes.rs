use minimap_core::services::ui::WaypointFilter;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::draw::WorldMapFrame;
use minimap_core::structs::view::CameraPose;
use minimap_core::structs::waypoint::{Waypoint, WaypointSource};

// the world map scenes the overlay benches and vertex counts use: a 1920 x 1080 frame centred on
// the origin, and 512 waypoints for the list
pub struct OverlayScenes;

impl OverlayScenes {
    pub const DISPLAY: (f32, f32) = (1920.0, 1080.0);
    // the zoom where the overlays first draw (6 px per chunk), and the closest zoom
    pub const GATE_ZOOM: f32 = 0.375;
    pub const MAX_ZOOM: f32 = 40.0;
    pub const WAYPOINTS: usize = 512;

    pub fn world_frame(zoom: f32) -> WorldMapFrame<'static> {
        WorldMapFrame {
            display: Self::DISPLAY,
            centre: (0.0, 0.0),
            zoom,
            camera: CameraPose { x: 0.0, z: 0.0, yaw: 0.0 },
            player_block: (0, 70, 0),
            font_size: 18.0,
            show_coordinates: false,
            show_nether_coordinates: false,
            dim: 0,
            angle: 0.0,
            regions: &[],
        }
    }

    // on a 64-block grid around the origin, as the waypoint draw benches place them
    pub fn waypoints() -> Vec<Waypoint> {
        let mut store = WaypointStore::new(Self::WAYPOINTS, 1);
        for i in 0..Self::WAYPOINTS as i32 {
            let pos = ((i % 32 - 16) * 64, 70, (i / 32 - 8) * 64);
            let _ = store.add("", 0, pos, WaypointSource::Hotkey, 0);
        }
        store.waypoints().to_vec()
    }

    pub fn rows(waypoints: &[Waypoint], query: &str) -> Vec<usize> {
        let mut filter = WaypointFilter::new();
        filter.set_query(query);
        let mut rows = Vec::new();
        filter.collect(waypoints, 0, &mut rows);
        rows
    }
}
