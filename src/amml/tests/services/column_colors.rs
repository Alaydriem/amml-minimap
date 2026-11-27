use std::collections::HashMap;

use alaydriem_minimap::services::{BiomeNames, BlockNames, ColumnColors};
use alaydriem_minimap::types::ColorSource;
use amml_api::testing::FakeGame;
use amml_api::{BlockTypeId, ColumnSample, Rgba};
use minimap_core::structs::Rgba as CoreRgba;

use crate::support::{FakeTerrain, MinimapRig, RecordingSink};

struct Colors {
    game: FakeGame,
    names: BlockNames,
    biomes: BiomeNames,
    sink: RecordingSink,
}

impl Colors {
    fn new() -> Self {
        let game = MinimapRig::game();
        game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
        let ctx = MinimapRig::ctx(&game);
        Self {
            names: BlockNames::new(ctx.blocks()),
            biomes: BiomeNames::new(ctx.biomes()),
            sink: RecordingSink::default(),
            game,
        }
    }

    fn convert(
        &mut self,
        sample: ColumnSample,
        vanilla: bool,
    ) -> (minimap_core::structs::chunk::ColumnSample, ColorSource) {
        let _ = &self.game;
        ColumnColors::convert(&sample, vanilla, &mut self.names, &mut self.biomes, &self.sink)
    }
}

fn packed(color: Rgba) -> u32 {
    CoreRgba::new(color.r, color.g, color.b, color.a).packed()
}

#[test]
fn a_colour_table_entry_wins_over_the_game_colour() {
    let mut colors = Colors::new();
    colors.sink.colors = vec![("minecraft:stone".to_string(), 0xFF11_2233)];

    let (sample, source) = colors.convert(FakeTerrain::stone(), true);

    assert_eq!(sample.surface, 0xFF11_2233);
    assert_eq!(source, ColorSource::Table);
}

#[test]
fn the_game_colour_shows_when_vanilla_colours_are_asked() {
    let mut colors = Colors::new();

    let (sample, source) = colors.convert(FakeTerrain::stone(), true);

    assert_eq!(sample.surface, packed(Rgba { r: 1, g: 2, b: 3, a: 255 }));
    assert_eq!(source, ColorSource::Vanilla);
}

#[test]
fn without_vanilla_colours_a_block_gets_the_fixed_colour_of_its_kind() {
    let mut colors = Colors::new();
    let water = ColumnSample { shown: FakeTerrain::WATER, ..FakeTerrain::stone() };

    let (stone, source) = colors.convert(FakeTerrain::stone(), false);
    let (water, _) = colors.convert(water, false);

    assert_eq!(stone.surface, ColumnColors::FALLBACK_BLOCK.packed());
    assert_eq!(water.surface, ColumnColors::FALLBACK_WATER.packed());
    assert_eq!(source, ColorSource::Fallback);
}

#[test]
fn a_block_the_game_gave_no_colour_for_gets_the_fixed_colour() {
    let mut colors = Colors::new();
    let sample = ColumnSample { shown_color: Rgba::default(), flags: ColumnSample::NO_COLOR, ..FakeTerrain::stone() };

    let (sample, source) = colors.convert(sample, true);

    assert_eq!(sample.surface, ColumnColors::FALLBACK_BLOCK.packed());
    assert_eq!(source, ColorSource::Fallback);
}

#[test]
fn a_void_column_takes_the_override_of_its_biome() {
    let mut colors = Colors::new();
    colors.sink.void_colors = HashMap::from([("plains".to_string(), 0xFF00_00AA)]);
    let void = ColumnSample {
        shown: BlockTypeId::NONE,
        shown_color: FakeTerrain::GRASS,
        flags: ColumnSample::VOID,
        ..FakeTerrain::stone()
    };

    let (sample, _) = colors.convert(void, true);

    assert_eq!(sample.surface, 0xFF00_00AA);
}

#[test]
fn a_void_column_without_an_override_shows_the_grass_colour_then_the_fixed_one() {
    let mut colors = Colors::new();
    let grass = ColumnSample {
        shown: BlockTypeId::NONE,
        shown_color: FakeTerrain::GRASS,
        flags: ColumnSample::VOID,
        ..FakeTerrain::stone()
    };
    let none = ColumnSample { shown_color: Rgba::default(), biome: ColumnSample::NO_BIOME, ..grass };

    let (grass, _) = colors.convert(grass, true);
    let (none, _) = colors.convert(none, true);

    assert_eq!(grass.surface, packed(FakeTerrain::GRASS));
    assert_eq!(none.surface, ColumnColors::FALLBACK_VOID.packed());
}

#[test]
fn transparent_water_puts_its_depth_in_the_alpha_byte_of_the_water_word() {
    let mut colors = Colors::new();
    let sample = ColumnSample {
        shown: FakeTerrain::SAND,
        water: FakeTerrain::WATER,
        water_color: Rgba { r: 10, g: 20, b: 30, a: 255 },
        water_depth: 5,
        ..FakeTerrain::stone()
    };

    let (sample, _) = colors.convert(sample, true);

    assert_eq!(sample.water, CoreRgba::new(10, 20, 30, 0).packed() | (5 << 24));
}

#[test]
fn a_column_without_water_has_a_zero_water_word() {
    let mut colors = Colors::new();

    let (sample, _) = colors.convert(FakeTerrain::stone(), true);

    assert_eq!(sample.water, 0);
}

#[test]
fn a_placeholder_and_an_empty_column_show_nothing() {
    let mut colors = Colors::new();
    let placeholder = ColumnSample { shown: FakeTerrain::PLACEHOLDER, ..FakeTerrain::stone() };
    let empty = ColumnSample { shown: BlockTypeId::NONE, flags: 0, ..FakeTerrain::stone() };

    let (placeholder, _) = colors.convert(placeholder, true);
    let (empty, source) = colors.convert(empty, true);

    assert_eq!(placeholder.surface, 0);
    assert_eq!(empty.surface, 0);
    assert_eq!(source, ColorSource::None);
}

#[test]
fn heights_retry_and_biome_come_from_the_sample() {
    let mut colors = Colors::new();
    let sample = ColumnSample { height: -12, solid: -20, flags: ColumnSample::RETRY, biome: 7, ..FakeTerrain::stone() };

    let (sample, _) = colors.convert(sample, true);

    assert_eq!((sample.height, sample.solid, sample.retry, sample.biome_id), (-12, -20, true, 7));
}
