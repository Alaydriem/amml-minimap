use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::chunk::{ChangeKinds, ChunkProbe, ChunkSurface, ColumnSample};

use crate::support::TestSurfaces;

#[test]
fn first_full_probe_is_a_first_load_with_every_change_kind() {
    let merge = ChunkSurface::empty().merge(&TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert!(merge.first_load);
    assert_eq!(merge.changes, ChangeKinds::ALL);
    assert_eq!(merge.surface.height_range(), Some((64, 64)));
    assert_eq!(merge.previous_range, None);
    assert!(!merge.needs_retry);
}

#[test]
fn skipped_columns_keep_their_stored_values() {
    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let mut probe = TestSurfaces::probe_flat(70, TestSurfaces::grey(50));
    probe.set_skipped(ChunkSurface::index(3, 4), true);
    let merge = stored.merge(&probe);
    assert_eq!(merge.surface.height[ChunkSurface::index(3, 4)], 64);
    assert_eq!(merge.surface.height[ChunkSurface::index(4, 4)], 70);
    assert!(merge.needs_retry);
    assert_eq!(merge.surface.height_range(), Some((70, 70)));
}

#[test]
fn change_kinds_name_only_what_changed() {
    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let colour_only = stored.merge(&TestSurfaces::probe_flat(64, TestSurfaces::grey(90)));
    assert_eq!(colour_only.changes, ChangeKinds::COLOUR);

    let mut solid_only = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    solid_only.solid[0] = 60;
    assert_eq!(stored.merge(&solid_only).changes, ChangeKinds::SOLID);

    let unchanged = stored.merge(&TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert!(unchanged.changes.is_empty());
    assert!(!unchanged.first_load);
}

#[test]
fn all_skipped_probe_of_unknown_chunk_stores_nothing() {
    let mut probe = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    probe.skipped = [u64::MAX; 4];
    let merge = ChunkSurface::empty().merge(&probe);
    assert!(!merge.surface.has_base_data());
    assert!(!merge.first_load);
    assert!(merge.changes.is_empty());
    assert!(merge.needs_retry);
}

#[test]
fn column_merge_needs_base_data_and_only_widens_the_range() {
    let sample = ColumnSample::new(TestSurfaces::grey(10), 0, 90, 90, false, 0);
    assert!(ChunkSurface::empty().merge_column(0, sample, None).is_none());

    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let merge = stored.merge_column(5, sample, None).expect("chunk has base data");
    assert_eq!(merge.changes, ChangeKinds::PIXELS);
    assert_eq!(merge.surface.height_range(), Some((64, 90)));
    let lower = ColumnSample::new(TestSurfaces::grey(100), 0, 64, 64, false, 0);
    let back = merge.surface.merge_column(5, lower, None).expect("chunk has base data");
    assert_eq!(back.surface.height_range(), Some((64, 90)));
}

#[test]
fn water_depth_and_colour_come_from_the_water_word() {
    let mut surface = ChunkSurface::empty();
    surface.water[7] = 0x0A_C8_10_20;
    assert_eq!(surface.water_depth(7), 10);
    let colour = surface.water_colour(7);
    assert_eq!((colour.r, colour.g, colour.b, colour.a), (0x20, 0x10, 0xC8, 255));
}

#[test]
fn a_rescan_that_only_names_the_biome_reports_a_biome_change() {
    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let mut incoming = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    incoming.biome.set(0, Some(BiomeKey(2)));
    let merge = stored.merge(&incoming);
    assert_eq!(merge.changes, ChangeKinds::BIOME);
    assert_eq!(merge.surface.biome.key(0), Some(BiomeKey(2)));
}

#[test]
fn an_unknown_incoming_biome_keeps_the_stored_one() {
    let mut first = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    first.biome.set(0, Some(BiomeKey(2)));
    let stored = ChunkSurface::empty().merge(&first).surface;
    let merge = stored.merge(&TestSurfaces::probe_flat(64, TestSurfaces::grey(100)));
    assert_eq!(merge.changes, ChangeKinds::NONE);
    assert_eq!(merge.surface.biome.key(0), Some(BiomeKey(2)));
}

#[test]
fn a_column_update_can_change_only_the_biome() {
    let stored = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    let column = ColumnSample::new(TestSurfaces::grey(100), 0, 64, 64, false, 0);
    let merge = stored.merge_column(5, column, Some(BiomeKey(3))).unwrap();
    assert_eq!(merge.changes, ChangeKinds::BIOME);
    assert_eq!(merge.surface.biome.key(5), Some(BiomeKey(3)));
    assert_eq!(stored.merge_column(5, column, None).unwrap().changes, ChangeKinds::NONE);
}

#[test]
fn the_surface_with_its_palette_is_3264_bytes() {
    assert_eq!(std::mem::size_of::<ChunkSurface>(), 3264);
    assert_eq!(std::mem::size_of::<ChunkProbe>(), 3776);
}

#[test]
fn a_first_load_takes_the_probed_palette_whole() {
    let mut probe = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    for c in 0..256 {
        probe.biome.set(c, Some(BiomeKey(if c < 128 { 1 } else { 4 })));
    }
    let merge = ChunkSurface::empty().merge(&probe);
    assert_eq!(merge.surface.biome, probe.biome);
    assert_eq!(merge.changes, ChangeKinds::ALL);
}

#[test]
fn a_rescan_with_the_same_palette_reports_no_biome_change() {
    let mut probe = TestSurfaces::probe_flat(64, TestSurfaces::grey(100));
    probe.biome.set(0, Some(BiomeKey(1)));
    let stored = ChunkSurface::empty().merge(&probe).surface;
    assert_eq!(stored.merge(&probe).changes, ChangeKinds::NONE);
}
