use minimap_core::services::biome::BiomeKeyTable;
use minimap_core::structs::biome::BiomeKey;

#[test]
fn keys_are_given_after_load_in_first_seen_order_and_the_table_holds_its_bound() {
    let mut table = BiomeKeyTable::new(3);
    assert_eq!(table.key_for("plains"), None);
    assert_eq!(table.table_id(), 0);
    table.mark_loaded(1, vec!["plains".into()]);
    assert_eq!(table.table_id(), 1);
    assert_eq!(table.key_for("plains"), Some(BiomeKey(0)));
    assert_eq!(table.key_for("forest"), Some(BiomeKey(1)));
    assert_eq!(table.name(BiomeKey(1)), Some("forest"));
    assert_eq!(table.take_dirty(), Some(vec!["plains".to_string(), "forest".to_string()]));
    assert_eq!(table.take_dirty(), None);
    assert_eq!(table.key_for("desert"), Some(BiomeKey(2)));
    assert_eq!(table.key_for("ocean"), None);
    assert_eq!(table.len(), 3);
}

#[test]
fn a_loaded_table_keeps_only_its_capacity_and_clear_unloads_it() {
    let mut table = BiomeKeyTable::new(2);
    table.mark_loaded(5, vec!["a".into(), "b".into(), "c".into()]);
    assert_eq!(table.len(), 2);
    assert_eq!(table.take_dirty(), None);
    table.clear();
    assert!(!table.is_loaded());
    assert_eq!(table.table_id(), 0);
    assert_eq!(table.key_for("a"), None);
}
