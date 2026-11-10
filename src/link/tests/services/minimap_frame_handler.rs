use std::sync::{Arc, OnceLock};

use amm_client::link::{FrameHandler, FrameInputView};
use amm_core::structs::frame::{CursorShape, FrameInputData, LayerKind};
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::{Context, RawInput, Rect};
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_link::services::MinimapFrameHandler;
use minimap_runtime::services::Runtime;
use minimap_runtime::structs::{RuntimePaths, ScanPlan, TickInput};

fn data(layer: LayerKind, time_secs: f64) -> FrameInputData<'static> {
    let allowed = if layer == LayerKind::Hud { FrameInputData::LAYER_HUD } else { FrameInputData::LAYER_OVERLAY };
    FrameInputData {
        width: 1280,
        height: 720,
        pixels_per_point: 1.0,
        time_secs,
        events: &[],
        allowed_layers: allowed,
        input_flags: 0,
    }
}

// one pass of `layer` through the handler in a amml-ui context, the way amm-client runs it
fn pass(
    handler: &mut MinimapFrameHandler,
    ctx: &Context,
    layer: LayerKind,
    time_secs: f64,
) -> amm_client::link::FrameReply {
    let mut reply = None;
    let mut output = ctx.run_pass(RawInput::default(), |ctx| {
        reply = Some(handler.frame(ctx, &FrameInputView::new(data(layer, time_secs), layer)));
    });
    output.textures_delta.clear();
    reply.expect("the pass ran the handler")
}

fn context() -> Context {
    let ctx = Context::default();
    Theme::install(&ctx, FontSource::BuiltIn);
    ctx
}

#[test]
fn a_frame_before_the_runtime_exists_is_empty() {
    let mut handler = MinimapFrameHandler::new(Arc::new(OnceLock::new()));

    let reply = pass(&mut handler, &context(), LayerKind::Hud, 0.0);

    assert_eq!(reply.layer_rect, Rect::NOTHING);
    assert_eq!(reply.cursor, CursorShape::Default);
}

#[test]
fn textures_lost_before_the_runtime_exists_does_nothing() {
    let mut handler = MinimapFrameHandler::new(Arc::new(OnceLock::new()));

    handler.textures_lost();

    assert_eq!(pass(&mut handler, &context(), LayerKind::Overlay, 0.0).layer_rect, Rect::NOTHING);
}

#[test]
fn the_hud_returns_the_corner_map_and_the_closed_overlay_nothing() {
    let dir = tempfile::tempdir().unwrap();
    let grant = TextureGrant { bytes: 64 * 1024 * 1024, count: 1024 };
    let runtime =
        Arc::new(Runtime::new(Config::default(), RuntimePaths { data_dir: dir.path().join("d") }, grant).unwrap());
    let player = PlayerSnapshot { present: true, x: 8.5, y: 80.0, z: 8.5, yaw: 0.0, dim: 0 };
    runtime.tick(&TickInput { player, mouse_grabbed: true, ..TickInput::default() }, &mut ScanPlan::default());
    let slot = Arc::new(OnceLock::new());
    let _ = slot.set(Arc::clone(&runtime));
    let mut handler = MinimapFrameHandler::new(slot);

    let hud = pass(&mut handler, &context(), LayerKind::Hud, 1.0);
    let overlay = pass(&mut handler, &context(), LayerKind::Overlay, 1.0);

    assert!(hud.layer_rect.is_positive(), "{hud:?}");
    assert_eq!(overlay.layer_rect, Rect::NOTHING);
    runtime.shutdown();
}
