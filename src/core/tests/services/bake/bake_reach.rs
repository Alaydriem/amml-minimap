use minimap_core::services::bake::{BakeReach, ShadowCaster};
use minimap_core::structs::LightDirection;

#[test]
fn the_reach_extends_toward_the_light_and_one_chunk_elsewhere() {
    let rays = (ShadowCaster::MAX_STEPS + 15) / 16 + 1;
    // azimuth 0: the sun lies toward negative z, straight above the x axis
    assert_eq!(BakeReach::offsets(LightDirection::new(0.0, 45.0)), (-1, 1, -rays, 1));
    // azimuth 90: the sun lies toward positive x
    let (dx0, dx1, _, _) = BakeReach::offsets(LightDirection::new(90.0, 45.0));
    assert_eq!((dx0, dx1), (-1, rays));
}
