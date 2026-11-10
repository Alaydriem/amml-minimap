use minimap_core::services::bake::WaterOverlay;
use minimap_core::structs::Rgba;
use minimap_core::structs::chunk::ChunkSurface;

fn with_water(depth: u8, rgb: (u8, u8, u8)) -> ChunkSurface {
    let mut surface = ChunkSurface::empty();
    surface.water[0] = Rgba::new(rgb.0, rgb.1, rgb.2, depth).packed();
    surface
}

#[test]
fn opacity_grows_with_depth() {
    let mut colours = [Rgba::new(100, 100, 100, 255); 256];
    WaterOverlay::apply(&with_water(5, (0, 0, 200)), 0.1, 0.6, &mut colours);
    assert_eq!(colours[0], Rgba::new(50, 50, 150, 255));
    assert_eq!(colours[1], Rgba::new(100, 100, 100, 255));
}

#[test]
fn opacity_is_capped_at_the_maximum() {
    let mut colours = [Rgba::new(0, 0, 0, 255); 256];
    WaterOverlay::apply(&with_water(200, (200, 200, 200)), 0.1, 0.6, &mut colours);
    assert_eq!(colours[0], Rgba::new(120, 120, 120, 255));
}
