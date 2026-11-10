use minimap_core::config::TerrainConfig;
use minimap_core::services::bake::{ChunkBaker, RegionBaker};
use minimap_core::structs::bake::{BakeOutput, BakeSettings, RegionBakeOutput};
use minimap_core::structs::pos::{ChunkPos, RegionPos};

use crate::support::{FakeEnvironment, TestSurfaces};

// hilly chunks across a region edge, so shadows and PCF cross chunk and region borders
fn hilly_environment() -> FakeEnvironment {
    let mut env = FakeEnvironment::new();
    for z in -1..3 {
        for x in -1..3 {
            let mut surface = TestSurfaces::stored_flat(64, TestSurfaces::grey(120));
            for i in 0..16 {
                let h = 64 + ((x * 7 + z * 3 + i as i32) % 9) as i16 * 4;
                surface = TestSurfaces::with_height(surface, i, (i * 5) % 16, h);
            }
            env = env.with_surface(ChunkPos::new(x, z, 0), surface);
        }
    }
    env
}

#[test]
fn a_region_bake_matches_chunk_bakes_that_see_the_same_neighbour_masks() {
    let env = hilly_environment();
    let settings = BakeSettings::new(&TerrainConfig::default());
    let region = RegionPos::new(0, 0, 0);
    let mut region_out = RegionBakeOutput::new();
    RegionBaker::new().bake(region, &env, &settings, &mut region_out);

    let mut with_masks = FakeEnvironment::new();
    with_masks.surfaces = env.surfaces.clone();
    for z in 0..16 {
        for x in 0..16 {
            let i = (z * 16 + x) as usize;
            if region_out.present[i] {
                with_masks.masks.insert(ChunkPos::new(x, z, 0), region_out.masks[i].clone());
            }
        }
    }
    let mut baker = ChunkBaker::new();
    for z in 0..2 {
        for x in 0..2 {
            let pos = ChunkPos::new(x, z, 0);
            let mut out = BakeOutput::new();
            baker.bake(pos, &env.surfaces[&pos], &with_masks, &settings, &mut out);
            let i = (z * 16 + x) as usize;
            assert!(region_out.present[i]);
            assert_eq!(out.mask, region_out.masks[i], "mask of {pos:?}");
            assert_eq!(out.pixels, region_out.pixels[i], "pixels of {pos:?}");
        }
    }
}

#[test]
fn chunks_without_base_data_are_not_present_in_the_output() {
    let env = hilly_environment();
    let mut out = RegionBakeOutput::new();
    RegionBaker::new().bake(RegionPos::new(0, 0, 0), &env, &BakeSettings::new(&TerrainConfig::default()), &mut out);
    assert!(out.present[0]);
    assert!(!out.present[16 * 5 + 5]);
}
