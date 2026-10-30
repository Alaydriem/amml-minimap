use std::collections::BTreeSet;

use alaydriem_minimap::MinimapMod;
use alaydriem_minimap::types::{MinimapEntry, MinimapFeature};
use amml_game::{EntryId, GameError, MaterialLayout};

use crate::support::{FakeResolver, FakeWorld};

#[test]
fn without_any_minimap_entry_every_feature_is_off_with_one_line_each() {
    let entries = MinimapMod::resolve(&FakeResolver::new(&FakeWorld::required())).unwrap();

    let off: BTreeSet<&str> = entries.off_features().into_iter().map(|(feature, _)| feature).collect();
    let lines = MinimapMod::off_lines(&entries);

    assert_eq!(off, MinimapFeature::ALL.into_iter().collect());
    assert_eq!(lines.len(), 5);
    for feature in MinimapFeature::ALL {
        assert_eq!(lines.iter().filter(|line| line.starts_with(feature)).count(), 1, "{feature}: {lines:?}");
    }
}

#[test]
fn a_missing_required_entry_refuses_the_enable() {
    let values: Vec<_> =
        FakeWorld::required().into_iter().filter(|(id, _)| *id != EntryId::LEVEL_CHUNK_GET_BLOCK).collect();

    let result = MinimapMod::resolve(&FakeResolver::new(&values));

    assert!(matches!(result, Err(GameError::MissingEntry { id }) if id == "LevelChunk.getBlock"));
}

#[test]
fn a_listener_slot_outside_the_table_turns_block_changes_off() {
    let mut values = FakeWorld::required();
    values.extend(
        FakeWorld::optional()
            .into_iter()
            .map(|(id, value)| if id.as_str() == "BlockSourceListener.onBlockChanged" { (id, 9) } else { (id, value) }),
    );
    let entries = MinimapMod::resolve(&FakeResolver::new(&values)).unwrap();

    let lines = MinimapMod::off_lines(&entries);

    assert!(lines.iter().any(|line| line.starts_with(MinimapFeature::BLOCK_CHANGES) && line.contains("slot 9")));
}

#[test]
fn a_missing_connection_destructor_keeps_the_server_identity_on_with_one_line() {
    let destructor = MinimapEntry::GAME_CONNECTION_INFO_DTOR;
    let values: Vec<_> =
        FakeWorld::required().into_iter().chain(FakeWorld::optional()).filter(|(id, _)| *id != destructor).collect();
    let entries = MinimapMod::resolve(&FakeResolver::new(&values)).unwrap();

    let lines = MinimapMod::off_lines(&entries);

    assert!(MinimapFeature::is_on(&entries, MinimapFeature::SERVER_IDENTITY));
    assert_eq!(lines.iter().filter(|line| line.contains(destructor.as_str())).count(), 1, "{lines:?}");
    assert!(!lines.iter().any(|line| line.starts_with(MinimapFeature::SERVER_IDENTITY)), "{lines:?}");
}

// the fake world with its block-material layout entry replaced by `material`
fn with_material(material: &[(EntryId, u64)]) -> Vec<(EntryId, u64)> {
    let mut values: Vec<_> =
        FakeWorld::required().into_iter().filter(|(id, _)| *id != EntryId::BLOCK_TYPE_MATERIAL_INLINE).collect();
    values.extend_from_slice(material);
    values
}

#[test]
fn the_pointer_material_layout_enables_without_a_material_line() {
    let values = with_material(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 0)]);
    let entries = MinimapMod::resolve(&FakeResolver::new(&values)).unwrap();

    assert!(matches!(MaterialLayout::of(&entries), Ok(MaterialLayout::Pointer)));
    assert!(!MinimapMod::off_lines(&entries).iter().any(|line| line.contains("block_material")));
}

#[test]
fn the_value_material_layout_enables() {
    let values = with_material(&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 1)]);
    let entries = MinimapMod::resolve(&FakeResolver::new(&values)).unwrap();

    assert!(matches!(MaterialLayout::of(&entries), Ok(MaterialLayout::Value)));
}

#[test]
fn an_unknown_or_missing_material_layout_refuses_the_enable() {
    for material in [&[(EntryId::BLOCK_TYPE_MATERIAL_INLINE, 2)][..], &[]] {
        let result = MinimapMod::resolve(&FakeResolver::new(&with_material(material)));

        assert!(matches!(result, Err(GameError::EntrySet { .. })), "{material:?}");
    }
}
