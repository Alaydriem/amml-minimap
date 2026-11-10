use crate::config::Config;

// which settings a running mod takes from a changed config: `live` at once, `world` when the next world
// starts; everything else waits for a restart. descriptor.hcl's `apply` modes say the same
pub struct ConfigMerge;

impl ConfigMerge {
    pub fn live(current: &Config, new: &Config) -> Config {
        let mut out = current.clone();
        out.font_size = new.font_size;
        out.mini_map = new.mini_map.clone();
        out.smooth_camera = new.smooth_camera.clone();
        out.keys = new.keys.clone();
        out.terrain.enable = new.terrain.enable;
        let (world, next) = (&mut out.world_map, &new.world_map);
        world.show_coordinates = next.show_coordinates;
        world.show_nether_coordinates = next.show_nether_coordinates;
        world.rotate_with_player = next.rotate_with_player;
        world.show_chunk_grid = next.show_chunk_grid;
        world.show_slime_chunks = next.show_slime_chunks;
        world.min_zoom = next.min_zoom;
        world.max_zoom = next.max_zoom;
        world.zoom_step = next.zoom_step;
        world.waypoint_label_zoom = next.waypoint_label_zoom;
        let (waypoints, next) = (&mut out.waypoints, &new.waypoints);
        waypoints.show_on_minimap = next.show_on_minimap;
        waypoints.max_edge_arrows = next.max_edge_arrows;
        waypoints.label_scale = next.label_scale;
        out
    }

    pub fn world(current: &Config, new: &Config) -> Config {
        let mut out = Self::live(current, new);
        let (terrain, next) = (&mut out.terrain, &new.terrain);
        terrain.scan_radius = next.scan_radius;
        terrain.scan_max_chunks_per_frame = next.scan_max_chunks_per_frame;
        terrain.rescan_interval_frames = next.rescan_interval_frames;
        terrain.region_bake_threshold = next.region_bake_threshold;
        terrain.enable_disk_cache = next.enable_disk_cache;
        out.cave_mode = new.cave_mode.clone();
        out.export.max_pixels = new.export.max_pixels;
        out.waypoints.enabled = new.waypoints.enabled;
        out
    }
}
