use minimap_core::structs::draw::{DrawList, TextureOp};
use minimap_ui::map::{DrawListShapes, LayerTextures};
use minimap_ui::testing::{HarnessEvent, HarnessInput, PassOutput, UiHarness};

// builds pass inputs and paints draw lists through a harness for the tests
pub struct Frames;

impl Frames {
    pub const DISPLAY: (f32, f32) = (1280.0, 720.0);

    pub fn input(cursor: (f32, f32), left_down: bool, events: &[HarnessEvent]) -> HarnessInput<'_> {
        HarnessInput::new(Self::DISPLAY, cursor, left_down, events)
    }

    // a 256 x 256 upload of opaque grey for texture `tex`, with its pixels in `buffers`
    pub fn upload(tex: u32, buffers: &mut Vec<Vec<u8>>) -> TextureOp {
        buffers.push(vec![128; 256 * 256 * 4]);
        TextureOp::Upload { tex, width: 256, height: 256, buffer: buffers.len() as u32 - 1 }
    }

    // one pass painting `list` with `textures` uploaded first, at `pixels_per_point`
    pub fn paint(list: &DrawList, textures: &[u32], pixels_per_point: f32) -> (PassOutput, LayerTextures) {
        let mut harness = UiHarness::new();
        let mut layer = LayerTextures::new(16);
        let mut buffers = Vec::new();
        let ops: Vec<TextureOp> = textures.iter().map(|tex| Self::upload(*tex, &mut buffers)).collect();
        let mut out = PassOutput::new();
        let input = HarnessInput { pixels_per_point, ..Self::input((0.0, 0.0), false, &[]) };
        let mut shapes = DrawListShapes::new();
        harness.run(&input, &mut out, |ctx| {
            layer.ensure_context(ctx);
            layer.apply(ctx, &ops, &buffers);
            shapes.paint(list, &ctx.layer_painter(egui::LayerId::background()), ctx.pixels_per_point(), &layer);
        });
        (out, layer)
    }
}
