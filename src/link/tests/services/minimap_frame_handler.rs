use std::sync::{Arc, OnceLock};

use amm_client::link::{FrameHandler, FrameInputView};
use amm_core::structs::frame::{CursorShape, FrameInputData, LayerKind};
use amml_ui::font::FontSource;
use amml_ui::theme::Theme;
use egui::{Context, RawInput, Rect};
use minimap_core::config::Config;
use minimap_core::structs::draw::TextureGrant;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_link::services::{KeyFeed, MinimapFrameHandler};
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
    let mut handler = MinimapFrameHandler::new(Arc::new(OnceLock::new()), Arc::new(KeyFeed::new()));

    let reply = pass(&mut handler, &context(), LayerKind::Hud, 0.0);

    assert_eq!(reply.layer_rect, Rect::NOTHING);
    assert_eq!(reply.cursor, CursorShape::Default);
}

#[test]
fn textures_lost_before_the_runtime_exists_does_nothing() {
    let mut handler = MinimapFrameHandler::new(Arc::new(OnceLock::new()), Arc::new(KeyFeed::new()));

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
    let mut handler = MinimapFrameHandler::new(slot, Arc::new(KeyFeed::new()));

    let hud = pass(&mut handler, &context(), LayerKind::Hud, 1.0);
    let overlay = pass(&mut handler, &context(), LayerKind::Overlay, 1.0);

    assert!(hud.layer_rect.is_positive(), "{hud:?}");
    assert_eq!(overlay.layer_rect, Rect::NOTHING);
    runtime.shutdown();
}

#[test]
fn the_overlay_pass_gets_the_fed_keys_and_the_hud_pass_does_not() {
    let keys = Arc::new(KeyFeed::new());
    keys.set_armed(true);
    keys.on_key(amml_api::KeyCode::ESCAPE, true, false, amml_api::Modifiers::NONE, true);
    let mut handler = MinimapFrameHandler::new(Arc::new(OnceLock::new()), Arc::clone(&keys));
    let mut hud = RawInput::default();
    let mut overlay = RawInput::default();

    handler.adjust_input(LayerKind::Hud, &mut hud);
    handler.adjust_input(LayerKind::Overlay, &mut overlay);

    assert!(hud.events.is_empty());
    assert!(
        overlay
            .events
            .iter()
            .any(|event| matches!(event, egui::Event::Key { key: egui::Key::Escape, pressed: true, .. }))
    );
}

// one pass of `layer` in which a widget of the layer has the keyboard focus when `focused`
fn pass_with_focus(handler: &mut MinimapFrameHandler, ctx: &Context, layer: LayerKind, focused: bool) {
    let mut output = ctx.run_pass(RawInput::default(), |ctx| {
        if focused {
            ctx.memory_mut(|memory| memory.request_focus(minimap_ui::id::MinimapUiId::DialogName.id()));
        }
        handler.frame(ctx, &FrameInputView::new(data(layer, 1.0), layer));
    });
    output.textures_delta.clear();
}

// whether a blocked Backspace reaches the next Overlay pass: the feed passes it only while typing
fn backspace_fed(keys: &KeyFeed) -> bool {
    keys.on_key(amml_api::KeyCode(0x08), true, false, amml_api::Modifiers::NONE, true);
    let mut raw = RawInput::default();
    keys.drain_into(&mut raw);
    raw.events.iter().any(|event| matches!(event, egui::Event::Key { key: egui::Key::Backspace, .. }))
}

#[test]
fn the_typing_flag_follows_the_keyboard_wish_of_the_overlay_pass_only() {
    let dir = tempfile::tempdir().unwrap();
    let grant = TextureGrant { bytes: 64 * 1024 * 1024, count: 1024 };
    let runtime =
        Arc::new(Runtime::new(Config::default(), RuntimePaths { data_dir: dir.path().join("d") }, grant).unwrap());
    let slot = Arc::new(OnceLock::new());
    let _ = slot.set(Arc::clone(&runtime));
    let keys = Arc::new(KeyFeed::new());
    keys.set_armed(true);
    let mut handler = MinimapFrameHandler::new(slot, Arc::clone(&keys));
    let ctx = context();

    pass_with_focus(&mut handler, &ctx, LayerKind::Hud, true);
    let after_hud = backspace_fed(&keys);
    pass_with_focus(&mut handler, &ctx, LayerKind::Overlay, true);
    let after_focused_overlay = backspace_fed(&keys);
    pass_with_focus(&mut handler, &context(), LayerKind::Overlay, false);
    let after_free_overlay = backspace_fed(&keys);

    assert!(!after_hud);
    assert!(after_focused_overlay);
    assert!(!after_free_overlay);
    runtime.shutdown();
}
