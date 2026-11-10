use minimap_core::config::TerrainConfig;
use minimap_core::services::bake::ChunkBaker;
use minimap_core::structs::Rgba;
use minimap_core::structs::bake::{BakeOutput, BakeSettings, RenderStyle};
use minimap_core::structs::chunk::{ChunkSurface, ShadowMask};
use minimap_core::structs::pos::ChunkPos;

use crate::support::{FakeEnvironment, TestSurfaces};

fn settings(style: RenderStyle) -> BakeSettings {
    BakeSettings { style, ..BakeSettings::new(&TerrainConfig::default()) }
}

#[test]
fn flat_style_outputs_the_surface_colours_with_water_applied() {
    let mut center = TestSurfaces::stored_flat(64, TestSurfaces::grey(100));
    center.water[0] = Rgba::new(0, 0, 200, 5).packed();
    let mut out = BakeOutput::new();
    ChunkBaker::new().bake(
        ChunkPos::new(0, 0, 0),
        &center,
        &FakeEnvironment::new(),
        &settings(RenderStyle::Flat),
        &mut out,
    );
    assert_eq!(Rgba::from_packed(out.pixels[0]), Rgba::new(50, 50, 150, 255));
    assert_eq!(Rgba::from_packed(out.pixels[1]), Rgba::new(100, 100, 100, 255));
    assert_eq!(out.mask.scale(), 0);
}

#[test]
fn shadow_map_style_darkens_the_column_behind_a_pillar() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 5, 5, 100);
    let mut out = BakeOutput::new();
    ChunkBaker::new().bake(
        ChunkPos::new(0, 0, 0),
        &center,
        &FakeEnvironment::new(),
        &settings(RenderStyle::ShadowMap),
        &mut out,
    );
    assert_eq!(out.mask.scale(), 2);
    let shadowed = Rgba::from_packed(out.pixels[ChunkSurface::index(7, 7)]);
    let lit = Rgba::from_packed(out.pixels[ChunkSurface::index(1, 1)]);
    assert!(shadowed.r < lit.r, "{shadowed:?} vs {lit:?}");
}

#[test]
fn a_soft_bake_from_the_stored_mask_matches_the_full_bake() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 5, 5, 100);
    let env = FakeEnvironment::new();
    let shadow = settings(RenderStyle::ShadowMap);
    let mut full = BakeOutput::new();
    let mut baker = ChunkBaker::new();
    baker.bake(ChunkPos::new(0, 0, 0), &center, &env, &shadow, &mut full);
    let mut soft = BakeOutput::new();
    baker.bake_soft(ChunkPos::new(0, 0, 0), &center, &full.mask, &env, &shadow, &mut soft);
    assert_eq!(soft.pixels, full.pixels);
    assert_eq!(soft.mask, full.mask);
}

#[test]
fn a_soft_bake_with_a_mask_at_another_scale_falls_back_to_the_full_bake() {
    let center = TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(100)), 5, 5, 100);
    let mut out = BakeOutput::new();
    ChunkBaker::new().bake_soft(
        ChunkPos::new(0, 0, 0),
        &center,
        &ShadowMask::new(4),
        &FakeEnvironment::new(),
        &settings(RenderStyle::ShadowMap),
        &mut out,
    );
    assert_eq!(out.mask.scale(), 2);
}
