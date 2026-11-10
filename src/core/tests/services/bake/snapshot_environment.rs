use minimap_core::config::TerrainConfig;
use minimap_core::services::bake::{BakeReach, ChunkBaker, SnapshotEnvironment};
use minimap_core::structs::bake::{BakeOutput, BakeSettings, RenderStyle};
use minimap_core::structs::chunk::ShadowMask;
use minimap_core::structs::pos::ChunkPos;
use minimap_core::traits::BakeEnvironment;

use crate::support::{FakeEnvironment, TestSurfaces};

fn hilly_world() -> FakeEnvironment {
    let mut env = FakeEnvironment::new();
    for cz in -8_i32..=8 {
        for cx in -8..=8 {
            let height = 64 + ((cx * 7 + cz * 13).rem_euclid(5) as i16) * 20;
            let surface =
                TestSurfaces::with_height(TestSurfaces::stored_flat(64, TestSurfaces::grey(120)), 3, 3, height);
            env = env.with_surface(ChunkPos::new(cx, cz, 0), surface);
        }
    }
    env
}

#[test]
fn a_bake_from_the_snapshot_matches_a_bake_from_the_full_world() {
    let world = hilly_world();
    let settings = BakeSettings { style: RenderStyle::ShadowMap, ..BakeSettings::new(&TerrainConfig::default()) };
    let pos = ChunkPos::new(0, 0, 0);
    let surface = *world.surface(pos).unwrap();
    let mut direct = BakeOutput::new();
    ChunkBaker::new().bake(pos, &surface, &world, &settings, &mut direct);

    let (dx0, dx1, dz0, dz1) = BakeReach::offsets(settings.light());
    let mut snapshot = SnapshotEnvironment::new();
    snapshot.capture_area(&world, (dx0, dx1), (dz0, dz1), 0);
    let mut copied = BakeOutput::new();
    ChunkBaker::new().bake(pos, &surface, &snapshot, &settings, &mut copied);

    assert_eq!(direct.pixels, copied.pixels);
    assert_eq!(direct.mask, copied.mask);
}

#[test]
fn only_chunks_with_data_are_copied_and_masks_come_along() {
    let pos = ChunkPos::new(2, 2, 0);
    let world = FakeEnvironment::new()
        .with_surface(pos, TestSurfaces::stored_flat(70, TestSurfaces::grey(90)))
        .with_mask(pos, ShadowMask::new(2));
    let mut snapshot = SnapshotEnvironment::new();
    snapshot.capture_area(&world, (0, 4), (0, 4), 0);
    assert_eq!(snapshot.len(), 1);
    assert_eq!(snapshot.surface(pos), world.surface(pos));
    assert_eq!(snapshot.mask(pos).map(ShadowMask::scale), Some(2));
    assert!(snapshot.surface(ChunkPos::new(1, 1, 0)).is_none());
}

#[test]
fn clearing_forgets_every_chunk_and_slots_are_reused_without_stale_masks() {
    let a = ChunkPos::new(0, 0, 0);
    let b = ChunkPos::new(1, 0, 0);
    let with_mask = FakeEnvironment::new()
        .with_surface(a, TestSurfaces::stored_flat(70, TestSurfaces::grey(90)))
        .with_mask(a, ShadowMask::new(2));
    let without_mask = FakeEnvironment::new().with_surface(b, TestSurfaces::stored_flat(80, TestSurfaces::grey(60)));
    let mut snapshot = SnapshotEnvironment::new();
    snapshot.capture(&with_mask, a);
    snapshot.clear();
    assert!(snapshot.is_empty());
    assert!(snapshot.surface(a).is_none());
    snapshot.capture(&without_mask, b);
    assert_eq!(snapshot.surface(b), without_mask.surface(b));
    assert!(snapshot.mask(b).is_none());
}
