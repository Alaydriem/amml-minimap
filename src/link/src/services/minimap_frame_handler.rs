use std::sync::{Arc, OnceLock};
use std::time::Instant;

use amm_client::link::{FrameHandler, FrameInputView, FrameReply};
use amm_core::structs::frame::{CursorShape, LayerKind};
use egui::{Context, RawInput, Rect};
use minimap_core::structs::draw::MapLayer;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::LayerInput;

use crate::services::KeyFeed;

// the manager calls this once per drawn layer on the render thread; it hands each pass to the runtime and
// times the mod's share of each frame. `slot` holds the runtime once the session has made it; `keys` gives the
// Overlay layer the key presses the manager cannot route to it
pub struct MinimapFrameHandler {
    slot: Arc<OnceLock<Arc<Runtime>>>,
    keys: Arc<KeyFeed>,
    // the frame time of the passes being summed, and their sum in microseconds
    frame_time: Option<f64>,
    frame_us: u32,
}

impl MinimapFrameHandler {
    pub fn new(slot: Arc<OnceLock<Arc<Runtime>>>, keys: Arc<KeyFeed>) -> Self {
        Self { slot, keys, frame_time: None, frame_us: 0 }
    }

    // both layers of one frame carry the same time; a new time closes the last frame's sum
    fn add_time(&mut self, runtime: &Runtime, time_secs: f64, pass_us: u32) {
        if self.frame_time != Some(time_secs) {
            if self.frame_time.is_some() {
                runtime.record_render(self.frame_us);
            }
            self.frame_time = Some(time_secs);
            self.frame_us = 0;
        }
        self.frame_us = self.frame_us.saturating_add(pass_us);
    }
}

impl FrameHandler for MinimapFrameHandler {
    fn frame(&mut self, ctx: &Context, input: &FrameInputView) -> FrameReply {
        let Some(runtime) = self.slot.get().cloned() else {
            return FrameReply { cursor: CursorShape::Default, layer_rect: Rect::NOTHING };
        };
        let started = Instant::now();
        let data = input.data();
        let layer = match input.layer() {
            LayerKind::Hud => MapLayer::Hud,
            LayerKind::Overlay => MapLayer::Overlay,
        };
        let pass = LayerInput {
            layer,
            width: data.width,
            height: data.height,
            pixels_per_point: data.pixels_per_point,
            time_secs: data.time_secs,
        };
        let reply = runtime.paint_layer(ctx, &pass);
        if layer == MapLayer::Overlay {
            self.keys.set_typing(ctx.egui_wants_keyboard_input());
        }
        let pass_us = u32::try_from(started.elapsed().as_micros()).unwrap_or(u32::MAX);
        self.add_time(&runtime, data.time_secs, pass_us);
        let cursor = if reply.arrow_cursor { CursorShape::Arrow } else { CursorShape::Default };
        FrameReply { cursor, layer_rect: reply.rect }
    }

    fn adjust_input(&mut self, layer: LayerKind, raw: &mut RawInput) {
        if layer == LayerKind::Overlay {
            self.keys.drain_into(raw);
        }
    }

    fn textures_lost(&mut self) {
        if let Some(runtime) = self.slot.get() {
            runtime.textures_lost();
        }
    }
}
