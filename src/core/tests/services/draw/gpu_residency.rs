use minimap_core::services::draw::GpuResidency;
use minimap_core::structs::draw::{ResidencyDecision, TextureOp};
use minimap_core::structs::pos::RegionPos;

fn r(x: i32) -> RegionPos {
    RegionPos::new(x, 0, 0)
}

#[test]
fn a_new_tile_is_uploaded_once_then_drawn_until_it_changes() {
    let mut gpu = GpuResidency::new(8, 600, 4);
    let mut ops = Vec::new();
    gpu.begin_frame(1);
    let ResidencyDecision::Upload { tex, created: true } = gpu.decide(r(0), Some(3), &mut ops) else { panic!() };
    assert!(tex >= 2);
    gpu.uploaded(r(0), 3);
    gpu.begin_frame(2);
    assert_eq!(gpu.decide(r(0), Some(3), &mut ops), ResidencyDecision::Draw(tex));
    assert_eq!(gpu.decide(r(0), Some(4), &mut ops), ResidencyDecision::Upload { tex, created: false });
    assert_eq!(gpu.decide(r(0), None, &mut ops), ResidencyDecision::Draw(tex));
}

#[test]
fn new_textures_per_frame_are_limited() {
    let mut gpu = GpuResidency::new(8, 600, 2);
    let mut ops = Vec::new();
    gpu.begin_frame(1);
    assert!(matches!(gpu.decide(r(0), Some(1), &mut ops), ResidencyDecision::Upload { .. }));
    assert!(matches!(gpu.decide(r(1), Some(1), &mut ops), ResidencyDecision::Upload { .. }));
    assert_eq!(gpu.decide(r(2), Some(1), &mut ops), ResidencyDecision::Placeholder);
    assert_eq!(gpu.decide(r(3), None, &mut ops), ResidencyDecision::Placeholder);
}

#[test]
fn the_budget_evicts_the_least_recently_drawn_texture() {
    let mut gpu = GpuResidency::new(2, 600, 4);
    let mut ops = Vec::new();
    for (frame, x) in [(1, 0), (2, 1)] {
        gpu.begin_frame(frame);
        gpu.decide(r(x), Some(1), &mut ops);
    }
    gpu.begin_frame(3);
    assert!(matches!(gpu.decide(r(2), Some(1), &mut ops), ResidencyDecision::Upload { created: true, .. }));
    assert_eq!(ops.len(), 1);
    assert_eq!(gpu.len(), 2);
    assert!(matches!(gpu.decide(r(0), Some(1), &mut ops), ResidencyDecision::Upload { created: true, .. }));
}

#[test]
fn idle_textures_are_released_and_their_ids_reused() {
    let mut gpu = GpuResidency::new(8, 10, 4);
    let mut ops = Vec::new();
    gpu.begin_frame(1);
    let ResidencyDecision::Upload { tex, .. } = gpu.decide(r(0), Some(1), &mut ops) else { panic!() };
    gpu.begin_frame(20);
    gpu.release_idle(&mut ops);
    assert_eq!(ops, vec![TextureOp::Release { tex }]);
    assert!(gpu.is_empty());
    let ResidencyDecision::Upload { tex: again, .. } = gpu.decide(r(5), Some(1), &mut ops) else { panic!() };
    assert_eq!(again, tex);
}

#[test]
fn resident_bytes_never_pass_the_budget() {
    let mut gpu = GpuResidency::new(240, 600, 4096);
    let mut ops = Vec::new();
    for frame in 1..=10 {
        gpu.begin_frame(frame);
        for x in 0..200 {
            gpu.decide(RegionPos::new(x, frame as i32, 0), Some(1), &mut ops);
            assert!(gpu.resident_bytes() <= gpu.budget_bytes());
        }
    }
    assert_eq!(gpu.len(), 240);
    assert_eq!(gpu.budget_bytes(), 240 * GpuResidency::TILE_BYTES);
}

#[test]
fn unloaded_regions_release_their_texture() {
    let mut gpu = GpuResidency::new(8, 600, 4);
    let mut ops = Vec::new();
    gpu.begin_frame(1);
    let ResidencyDecision::Upload { tex: kept, .. } = gpu.decide(r(0), Some(1), &mut ops) else { panic!() };
    let ResidencyDecision::Upload { tex: gone, .. } = gpu.decide(r(1), Some(1), &mut ops) else { panic!() };
    ops.clear();

    gpu.release_unloaded(|region| region == r(0), &mut ops);

    assert_eq!(ops, [TextureOp::Release { tex: gone }]);
    assert_eq!(gpu.len(), 1);
    gpu.begin_frame(2);
    assert_eq!(gpu.decide(r(0), Some(1), &mut ops), ResidencyDecision::Upload { tex: kept, created: false });
}

#[test]
fn a_released_id_is_reused() {
    let mut gpu = GpuResidency::new(8, 600, 4);
    let mut ops = Vec::new();
    gpu.begin_frame(1);
    let ResidencyDecision::Upload { tex, .. } = gpu.decide(r(0), Some(1), &mut ops) else { panic!() };
    gpu.release_unloaded(|_| false, &mut ops);

    let again = gpu.decide(r(5), Some(1), &mut ops);

    assert_eq!(again, ResidencyDecision::Upload { tex, created: true });
}
