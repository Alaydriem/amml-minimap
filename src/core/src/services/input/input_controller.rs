use crate::config::Config;
use crate::services::draw::MapRotation;
use crate::structs::input::{KeyPhase, KeyResult, KeybindAction, KeybindInput};
use crate::structs::view::ViewState;

// keybind and pointer decisions: world map toggle, minimap toggle and zoom, world map wheel zoom and drag
pub struct InputController {
    waypoints_enabled: bool,
    min_radius: i32,
    max_radius: i32,
    zoom_step: f32,
    min_zoom: f32,
    max_zoom: f32,
}

impl InputController {
    pub fn new(config: &Config) -> Self {
        Self {
            waypoints_enabled: config.waypoints.enabled,
            min_radius: config.mini_map.min_radius_chunks,
            max_radius: config.mini_map.max_radius_chunks,
            zoom_step: config.world_map.zoom_step,
            min_zoom: config.world_map.min_zoom,
            max_zoom: config.world_map.max_zoom,
        }
    }

    // on the closed map a keybind acts only while the game holds the mouse, so typing in chat is left alone;
    // zoom acts on the key's repeats too, every other action on the press only
    pub fn on_keybind(&self, view: &mut ViewState, input: KeybindInput) -> KeyResult {
        let mut result = KeyResult::default();
        let acts = match input.phase {
            KeyPhase::Pressed => true,
            KeyPhase::Repeat => matches!(input.action, KeybindAction::ZoomIn | KeybindAction::ZoomOut),
            KeyPhase::Released => false,
        };
        if !acts || (!view.world_map_open && !input.mouse_grabbed) {
            return result;
        }
        match input.action {
            KeybindAction::ToggleWorldMap if view.world_map_open => Self::close_world_map(view),
            KeybindAction::ToggleWorldMap => Self::open_world_map(view),
            KeybindAction::ToggleMinimap => view.show_minimap = !view.show_minimap,
            KeybindAction::ZoomIn | KeybindAction::ZoomOut if !view.world_map_open => {
                let step = if input.action == KeybindAction::ZoomIn { -1 } else { 1 };
                let radius = (view.minimap_radius + step).clamp(self.min_radius, self.max_radius);
                result.radius_changed = radius != view.minimap_radius;
                view.minimap_radius = radius;
            }
            KeybindAction::AddWaypoint => result.add_waypoint = self.waypoints_enabled && !view.world_map_open,
            KeybindAction::ToggleCave => result.toggle_cave = true,
            KeybindAction::ToggleCoordinates => {
                view.show_coordinates = !view.show_coordinates;
                result.coordinates_changed = true;
            }
            KeybindAction::ZoomIn | KeybindAction::ZoomOut => {}
        }
        result
    }

    pub fn close_world_map(view: &mut ViewState) {
        view.world_map_open = false;
        view.anchor = None;
    }

    fn open_world_map(view: &mut ViewState) {
        view.world_map_open = true;
        view.offset_x = 0.0;
        view.offset_z = 0.0;
        view.anchor = None;
    }

    // `lines` wheel notches over the open map, away from the player positive
    pub fn on_wheel(&self, view: &mut ViewState, lines: f32) {
        if !view.world_map_open || lines == 0.0 {
            return;
        }
        view.zoom = (view.zoom * self.zoom_step.powf(lines)).clamp(self.min_zoom, self.max_zoom);
    }

    // left-drag pan: the cursor moved by (dx, dy) screen pixels on a map turned by `angle` radians,
    // so the map moves under the cursor whatever the turn
    pub fn on_drag(&self, view: &mut ViewState, dx: f32, dy: f32, angle: f32) {
        let zoom = view.zoom.clamp(self.min_zoom, self.max_zoom);
        let (mx, mz) = MapRotation::new(angle, (0.0, 0.0)).invert_vector((dx, dy));
        if let Some((x, z)) = view.anchor {
            view.anchor = Some((x - f64::from(mx / zoom), z - f64::from(mz / zoom)));
            return;
        }
        view.offset_x -= mx / zoom;
        view.offset_z -= mz / zoom;
    }
}
