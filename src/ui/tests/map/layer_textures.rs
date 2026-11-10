use egui::TextureOptions;
use minimap_core::services::draw::GpuResidency;
use minimap_core::structs::draw::TextureOp;
use minimap_ui::map::LayerTextures;
use minimap_ui::testing::{PassOutput, UiHarness};

use crate::support::Frames;

// one pass that applies `ops` to `layer` in the harness's context; returns what the pass asked the manager for
fn pass(harness: &mut UiHarness, layer: &mut LayerTextures, ops: &[TextureOp], buffers: &[Vec<u8>]) -> PassOutput {
    let mut out = PassOutput::new();
    harness.run(&Frames::input((0.0, 0.0), false, &[]), &mut out, |ctx| {
        layer.ensure_context(ctx);
        layer.apply(ctx, ops, buffers);
    });
    out
}

#[test]
fn textures_load_into_the_layer_context_with_nearest_sampling_for_tiles_and_the_corner_map() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(4);
    let mut buffers = Vec::new();
    let ops = [Frames::upload(GpuResidency::MINIMAP_TEXTURE, &mut buffers), Frames::upload(7, &mut buffers)];

    let out = pass(&mut harness, &mut layer, &ops, &buffers);

    let set: Vec<_> = out.textures_set.iter().filter(|(_, size, _)| *size == [256, 256]).collect();
    assert_eq!(set.len(), 2);
    let options = |tex: u32| set.iter().find(|(id, _, _)| Some(*id) == layer.id(tex)).map(|(_, _, options)| *options);
    assert_eq!(options(GpuResidency::MINIMAP_TEXTURE), Some(TextureOptions::NEAREST));
    assert_eq!(options(7), Some(TextureOptions::NEAREST));
}

#[test]
fn an_upload_of_a_held_texture_updates_it_in_place() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(4);
    let mut buffers = Vec::new();
    let first = [Frames::upload(7, &mut buffers)];
    pass(&mut harness, &mut layer, &first, &buffers);
    let id = layer.id(7);

    let out = pass(&mut harness, &mut layer, &first, &buffers);

    assert_eq!(layer.id(7), id);
    assert_eq!(layer.len(), 1);
    assert!(out.textures_set.iter().any(|(set, _, _)| Some(*set) == id));
}

#[test]
fn a_new_context_clears_the_handles() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(4);
    let mut buffers = Vec::new();
    let ops = [Frames::upload(7, &mut buffers)];
    pass(&mut harness, &mut layer, &ops, &buffers);
    let mut same = true;
    harness.run(&Frames::input((0.0, 0.0), false, &[]), &mut PassOutput::new(), |ctx| {
        same = !layer.ensure_context(ctx);
    });

    harness.replace_context();
    let mut changed = false;
    harness.run(&Frames::input((0.0, 0.0), false, &[]), &mut PassOutput::new(), |ctx| {
        changed = layer.ensure_context(ctx);
    });

    assert!(same, "the same context keeps the handles");
    assert!(changed && layer.is_empty() && layer.id(7).is_none());
}

#[test]
fn a_release_drops_the_handle() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(4);
    let mut buffers = Vec::new();
    let ops = [Frames::upload(7, &mut buffers)];
    pass(&mut harness, &mut layer, &ops, &buffers);

    pass(&mut harness, &mut layer, &[TextureOp::Release { tex: 7 }], &buffers);

    assert!(layer.id(7).is_none() && layer.is_empty());
}

#[test]
fn uploads_past_the_capacity_are_refused() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(2);
    let mut buffers = Vec::new();
    let ops: Vec<TextureOp> = (2..6).map(|tex| Frames::upload(tex, &mut buffers)).collect();

    pass(&mut harness, &mut layer, &ops, &buffers);

    assert_eq!(layer.len(), 2);
    assert!(layer.id(2).is_some() && layer.id(3).is_some() && layer.id(4).is_none() && layer.id(5).is_none());
}

#[test]
fn an_upload_whose_pixels_do_not_match_its_size_is_skipped() {
    let mut harness = UiHarness::new();
    let mut layer = LayerTextures::new(2);
    let buffers = vec![vec![0u8; 16]];
    let ops = [TextureOp::Upload { tex: 7, width: 256, height: 256, buffer: 0 }];

    pass(&mut harness, &mut layer, &ops, &buffers);

    assert!(layer.is_empty());
}
