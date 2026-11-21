use amml_api::{BlockTypeId, ColumnSample as GameSample, Rgba as GameRgba};
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::ColumnSample;

use crate::services::{BiomeNames, BlockNames};
use crate::traits::MapSink;
use crate::types::{BlockKind, ColorSource};

// turns one column the loader read into the core column sample: the colour shown and the water word. A colour from
// the scan plan's table wins; then the game's map colour, when it was asked for and given; else a fixed colour per
// kind of block. A void column takes the biome's override, else the game's grass colour, else FALLBACK_VOID
pub struct ColumnColors;

impl ColumnColors {
    // the vanilla map colours of stone, water and grass, for when the game's are off or cannot be read
    pub const FALLBACK_BLOCK: Rgba = Rgba { r: 112, g: 112, b: 112, a: 255 };
    pub const FALLBACK_WATER: Rgba = Rgba { r: 64, g: 64, b: 255, a: 255 };
    pub const FALLBACK_VOID: Rgba = Rgba { r: 127, g: 178, b: 56, a: 255 };

    // `vanilla`: the request asked for the game's colours
    pub fn convert<S: MapSink>(
        sample: &GameSample,
        vanilla: bool,
        names: &mut BlockNames,
        biomes: &mut BiomeNames,
        sink: &S,
    ) -> (ColumnSample, ColorSource) {
        let (surface, source) = if sample.shown.is_none() {
            (Self::empty_color(sample, biomes, sink), ColorSource::None)
        } else {
            Self::block_color(sample.shown, sample.shown_color, vanilla, names, sink)
        };
        let water = Self::water_word(sample, vanilla, names, sink);
        let column =
            ColumnSample::new(surface.packed(), water, sample.height, sample.solid, sample.retry(), sample.biome);
        (column, source)
    }

    fn block_color<S: MapSink>(
        id: BlockTypeId,
        game: GameRgba,
        vanilla: bool,
        names: &mut BlockNames,
        sink: &S,
    ) -> (Rgba, ColorSource) {
        let name = names.get(id);
        let kind = name.map_or(BlockKind::Other, |name| name.kind);
        if kind == BlockKind::Placeholder {
            return (Rgba::TRANSPARENT, ColorSource::None);
        }
        let table = name.and_then(|name| {
            sink.block_colors().iter().find(|(entry, _)| entry.as_str() == &*name.name).map(|(_, colour)| *colour)
        });
        if let Some(colour) = table {
            return (Rgba::from_packed(colour), ColorSource::Table);
        }
        if vanilla && game.a != 0 {
            return (Self::opaque(game), ColorSource::Vanilla);
        }
        let fallback = if kind == BlockKind::Water { Self::FALLBACK_WATER } else { Self::FALLBACK_BLOCK };
        (fallback, ColorSource::Fallback)
    }

    // a column with no shown block: void (nothing but air) has a colour, anything else shows nothing
    fn empty_color<S: MapSink>(sample: &GameSample, biomes: &mut BiomeNames, sink: &S) -> Rgba {
        if sample.flags & GameSample::VOID == 0 {
            return Rgba::TRANSPARENT;
        }
        let fallback = if sample.shown_color.a != 0 { Self::opaque(sample.shown_color) } else { Self::FALLBACK_VOID };
        match biomes.get(sample.biome) {
            Some(name) => Rgba::from_packed(sink.resolve_void_color(name, fallback.packed())),
            None => fallback,
        }
    }

    // the water colour with the depth in the alpha byte; 0 without water
    fn water_word<S: MapSink>(sample: &GameSample, vanilla: bool, names: &mut BlockNames, sink: &S) -> u32 {
        if sample.water_depth == 0 {
            return 0;
        }
        let id = if sample.water.is_none() { sample.shown } else { sample.water };
        let (colour, _) = Self::block_color(id, sample.water_color, vanilla, names, sink);
        Rgba::new(colour.r, colour.g, colour.b, 0).packed() | (u32::from(sample.water_depth) << 24)
    }

    fn opaque(colour: GameRgba) -> Rgba {
        Rgba::new(colour.r, colour.g, colour.b, 255)
    }
}
