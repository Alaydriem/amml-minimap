use bytemuck::Zeroable;
use minimap_core::structs::biome::{BiomeKey, BiomePalette};

#[test]
fn a_zeroed_palette_reads_as_unknown() {
    let p = BiomePalette::zeroed();
    assert!((0..256).all(|c| p.key(c).is_none()));
    assert_eq!(std::mem::size_of::<BiomePalette>(), BiomePalette::BYTES);
    assert_eq!(BiomePalette::BYTES, 72);
    assert_eq!(BiomePalette::SLOTS, 3);
}

#[test]
fn one_biome_takes_one_slot() {
    let mut p = BiomePalette::empty();
    for c in 0..256 {
        assert!(p.set(c, Some(BiomeKey(7))));
    }
    assert_eq!(p.distinct(), 1);
    assert!((0..256).all(|c| p.key(c) == Some(BiomeKey(7))));
}

#[test]
fn three_biomes_fit_and_a_fourth_takes_the_first_biome() {
    let mut p = BiomePalette::empty();
    for c in 0..3 {
        assert!(p.set(c, Some(BiomeKey(c as u16 + 100))));
    }
    assert!(!p.set(3, Some(BiomeKey(999))));
    assert_eq!(p.key(3), Some(BiomeKey(100)));
    assert_eq!(p.key(2), Some(BiomeKey(102)));
    assert_eq!(p.distinct(), BiomePalette::SLOTS);
}

#[test]
fn a_column_update_reuses_a_slot_or_takes_one_nobody_uses() {
    let mut p = BiomePalette::empty();
    for c in 0..3 {
        p.set(c, Some(BiomeKey(c as u16)));
    }
    p.set(1, Some(BiomeKey(0)));
    assert_eq!(p.distinct(), 2);
    assert!(p.set(200, Some(BiomeKey(500))));
    assert_eq!(p.key(200), Some(BiomeKey(500)));
    assert_eq!(p.key(1), Some(BiomeKey(0)));
}

#[test]
fn setting_unknown_clears_a_column() {
    let mut p = BiomePalette::empty();
    p.set(9, Some(BiomeKey(2)));
    assert!(p.set(9, None));
    assert_eq!(p.key(9), None);
    assert_eq!(p.distinct(), 0);
}
