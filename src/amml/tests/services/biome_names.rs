use alaydriem_minimap::services::BiomeNames;
use amml_api::ColumnSample;

use crate::support::{FakeTerrain, MinimapRig};

#[test]
fn a_name_is_kept_until_the_world_changes() {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
    let mut biomes = BiomeNames::new(MinimapRig::ctx(&game).biomes());

    assert_eq!(biomes.get(FakeTerrain::PLAINS), Some("plains"));
    assert_eq!(biomes.len(), 1);

    biomes.clear();

    assert!(biomes.is_empty());
}

#[test]
fn no_biome_and_an_unknown_id_are_not_kept() {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
    let mut biomes = BiomeNames::new(MinimapRig::ctx(&game).biomes());

    assert_eq!(biomes.get(ColumnSample::NO_BIOME), None);
    assert_eq!(biomes.get(200), None);
    assert!(biomes.is_empty());
}
