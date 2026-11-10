use minimap_core::services::biome::HoverBiomeCache;
use minimap_core::structs::biome::{BiomeKey, BiomePalette, RegionBiomes};
use minimap_core::structs::pos::RegionPos;

fn biomes_with_key(key: u16) -> RegionBiomes {
    let mut palette = BiomePalette::empty();
    palette.set(0, Some(BiomeKey(key)));
    let mut biomes = RegionBiomes::empty();
    biomes.set((0, 0), palette);
    biomes
}

#[test]
fn the_cache_holds_its_capacity_and_drops_the_least_recently_used_region() {
    let mut cache = HoverBiomeCache::new(2);
    cache.insert(RegionPos::new(0, 0, 0), biomes_with_key(1));
    cache.insert(RegionPos::new(1, 0, 0), biomes_with_key(2));
    assert!(cache.lookup(0, 0, 0).is_some());
    cache.insert(RegionPos::new(2, 0, 0), biomes_with_key(3));
    assert_eq!(cache.len(), 2);
    assert_eq!(cache.capacity(), 2);
    assert!(cache.lookup(256, 0, 0).is_none());
    assert_eq!(cache.lookup(0, 0, 0), Some(Some(BiomeKey(1))));
}

#[test]
fn a_cached_region_without_data_answers_no_biome_at_negative_blocks() {
    let mut cache = HoverBiomeCache::new(16);
    cache.insert(RegionPos::new(-1, -1, 0), RegionBiomes::empty());
    assert!(cache.contains(RegionPos::new(-1, -1, 0)));
    assert_eq!(cache.lookup(-1, -1, 0), Some(None));
    assert_eq!(cache.lookup(-1, -1, 1), None);
}

#[test]
fn a_forgotten_region_is_no_longer_cached() {
    let mut cache = HoverBiomeCache::new(16);
    cache.insert(RegionPos::new(0, 0, 0), RegionBiomes::empty());
    cache.forget(RegionPos::new(0, 0, 0));
    assert_eq!(cache.lookup(0, 0, 0), None);
    assert_eq!(cache.len(), 0);
}

#[test]
fn a_zero_capacity_still_holds_one_region() {
    let mut cache = HoverBiomeCache::new(0);
    cache.insert(RegionPos::new(0, 0, 0), RegionBiomes::empty());
    assert_eq!(cache.len(), 1);
    cache.clear();
    assert_eq!(cache.len(), 0);
}
