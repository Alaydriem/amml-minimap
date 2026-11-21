use alaydriem_minimap::services::BlockNames;
use alaydriem_minimap::types::BlockKind;
use amml_api::BlockTypeId;

use crate::support::{FakeTerrain, MinimapRig};

#[test]
fn a_name_is_read_once_and_kept_with_its_kind() {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
    let mut names = BlockNames::new(MinimapRig::ctx(&game).blocks());

    let water = names.get(FakeTerrain::WATER).cloned().unwrap();
    game.set_client(MinimapRig::player(0, amml_api::testing::FakeWorld::default()));

    assert_eq!(&*water.name, "minecraft:water");
    assert_eq!(water.kind, BlockKind::Water);
    assert_eq!(names.get(FakeTerrain::WATER).map(|name| name.kind), Some(BlockKind::Water));
    assert!(names.get(BlockTypeId::NONE).is_none());
}

#[test]
fn a_new_id_past_the_capacity_empties_the_cache_first() {
    let game = MinimapRig::game();
    game.set_client(MinimapRig::player(0, FakeTerrain::world(&[])));
    let mut names = BlockNames::new(MinimapRig::ctx(&game).blocks());
    for id in 1..=BlockNames::CAPACITY as u32 {
        names.get(BlockTypeId(id));
    }
    assert_eq!(names.len(), BlockNames::CAPACITY);

    names.get(BlockTypeId(BlockNames::CAPACITY as u32 + 1));
    names.get(BlockTypeId(1));

    assert_eq!(names.len(), 2);
}
