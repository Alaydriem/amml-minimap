use minimap_bench::services::BenchTerrain;
use minimap_core::structs::pos::ChunkPos;

#[test]
fn the_terrain_is_the_same_on_every_call_and_has_hills_and_water() {
    let pos = ChunkPos::new(-3, 5, 0);
    assert_eq!(BenchTerrain::stored(pos), BenchTerrain::stored(pos));
    let chunks = BenchTerrain::chunks(ChunkPos::new(0, 0, 0), 4);
    assert_eq!(chunks.len(), 81);
    let heights: Vec<i16> = chunks.iter().flat_map(|(_, s)| s.solid.iter().copied()).collect();
    let (low, high) = (heights.iter().min().unwrap(), heights.iter().max().unwrap());
    assert!(*low < BenchTerrain::SEA_LEVEL, "no water: lowest {low}");
    assert!(high - low > 30, "too flat: {low}..{high}");
    // a shadow ray climbs about 166 blocks in 96 steps, so every hill is in reach
    assert!(high - low < 160);
    assert!(chunks.iter().all(|(_, s)| s.has_base_data()));
}

#[test]
fn a_water_column_keeps_its_depth_in_the_water_alpha() {
    let (x, z) = (-200..200)
        .flat_map(|x| (-200..200).map(move |z| (x, z)))
        .find(|&(x, z)| BenchTerrain::height(x, z) < BenchTerrain::SEA_LEVEL)
        .expect("some water near the origin");
    let column = BenchTerrain::column(x, z);
    assert_eq!(column.height, BenchTerrain::SEA_LEVEL);
    assert_eq!((column.water >> 24) as i16, BenchTerrain::SEA_LEVEL - column.solid);
}

#[test]
fn a_store_holds_every_chunk_it_is_given() {
    let chunks = BenchTerrain::chunks(ChunkPos::new(10, 10, 1), 3);
    let store = BenchTerrain::store(&chunks);
    assert_eq!(store.len(), chunks.len());
}
