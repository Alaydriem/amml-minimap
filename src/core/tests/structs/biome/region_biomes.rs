use minimap_core::structs::biome::{BiomeKey, BiomePalette, RegionBiomes};

#[test]
fn a_region_answers_only_for_chunks_it_holds() {
    let mut biomes = RegionBiomes::empty();
    assert!(biomes.is_empty());
    let mut palette = BiomePalette::empty();
    palette.set(5, Some(BiomeKey(2)));
    biomes.set((1, 15), palette);
    assert!(!biomes.is_empty());
    assert_eq!(biomes.key((1, 15), 5), Some(BiomeKey(2)));
    assert_eq!(biomes.key((1, 15), 6), None);
    assert_eq!(biomes.key((2, 15), 5), None);
}
