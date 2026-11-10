use minimap_core::services::biome::{BiomeKeyTable, BiomeResolver};
use minimap_core::structs::chunk::ChunkSurface;

use crate::support::TestSurfaces;

fn loaded() -> BiomeKeyTable {
    let mut table = BiomeKeyTable::new(BiomeKeyTable::CAPACITY);
    table.mark_loaded(1, Vec::new());
    table
}

#[test]
fn a_chunk_of_one_named_biome_gets_one_slot() {
    let mut table = loaded();
    let (mut resolver, mut surface) = (BiomeResolver::new(), TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    resolver.answer(1, "plains");
    resolver.fill(&[1; 256], &mut surface, &mut table);
    let plains = table.key_for("plains");
    assert!(plains.is_some());
    assert!((0..256).all(|c| surface.biome.key(c) == plains));
    assert_eq!(surface.biome.distinct(), 1);
}

#[test]
fn unnamed_ids_are_unknown_and_ask_for_a_name() {
    let mut table = loaded();
    let (mut resolver, mut surface) = (BiomeResolver::new(), TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert!(resolver.fill(&[7; 256], &mut surface, &mut table), "the chunk waits for a name");
    assert!((0..256).all(|c| surface.biome.key(c).is_none()));
    let mut out = Vec::new();
    resolver.take_requests(&mut out);
    assert_eq!(out, vec![7]);
}

#[test]
fn skipped_columns_and_an_unloaded_table_give_unknown() {
    let mut table = loaded();
    let mut resolver = BiomeResolver::new();
    resolver.answer(1, "plains");
    let mut surface = ChunkSurface::empty();
    resolver.fill(&[1; 256], &mut surface, &mut table);
    assert!((0..256).all(|c| surface.biome.key(c).is_none()));

    let mut unloaded = BiomeKeyTable::new(BiomeKeyTable::CAPACITY);
    let mut probed = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    resolver.fill(&[1; 256], &mut probed, &mut unloaded);
    assert!((0..256).all(|c| probed.biome.key(c).is_none()));
    assert_eq!(unloaded.len(), 0);
}

#[test]
fn a_new_table_does_not_reuse_keys_from_the_old_one() {
    let mut first = loaded();
    let mut resolver = BiomeResolver::new();
    resolver.answer(1, "plains");
    resolver.answer(2, "forest");
    let mut surface = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    resolver.fill(&[2; 256], &mut surface, &mut first);
    let mut second = BiomeKeyTable::new(BiomeKeyTable::CAPACITY);
    second.mark_loaded(2, vec!["plains".into()]);
    let mut again = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    resolver.fill(&[1; 256], &mut again, &mut second);
    assert_eq!(again.biome.key(0), second.key_for("plains"));
    assert_eq!(second.name(again.biome.key(0).unwrap()), Some("plains"));
}

#[test]
fn a_named_chunk_does_not_wait_and_an_unloaded_table_does() {
    let mut table = loaded();
    let mut resolver = BiomeResolver::new();
    resolver.answer(1, "plains");
    resolver.answer(2, "");
    let mut surface = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    let mut ids = [1u16; 256];
    ids[0] = 2;
    assert!(!resolver.fill(&ids, &mut surface, &mut table), "no name is a final answer");
    let mut unloaded = BiomeKeyTable::new(BiomeKeyTable::CAPACITY);
    let mut again = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    assert!(resolver.fill(&ids, &mut again, &mut unloaded));
}
