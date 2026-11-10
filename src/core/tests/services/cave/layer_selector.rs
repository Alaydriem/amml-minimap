use minimap_core::config::Config;
use minimap_core::services::cave::{LayerIds, LayerSelector};

fn selector() -> LayerSelector {
    LayerSelector::new(&Config::default())
}

#[test]
fn off_it_shows_the_surface_with_each_dimension_camera_height() {
    let mut layers = selector();
    let overworld = layers.select(0, 70);
    assert_eq!((overworld.id, overworld.band, overworld.camera_height), (0, None, 320));
    assert_eq!(overworld.scan_radius_chunks, Config::default().terrain.scan_radius / 16);
    assert_eq!(layers.select(1, 70).camera_height, 127);
    assert_eq!(layers.select(2, 70).camera_height, 256);
}

#[test]
fn on_it_shows_the_band_around_the_feet_with_the_camera_at_its_top() {
    let mut layers = selector();
    assert!(layers.toggle());
    let cave = layers.select(0, 37);
    assert_eq!(cave.band, Some(2));
    assert_eq!(cave.id, LayerIds::cave(0, 2));
    assert_eq!(cave.camera_height, 47);
    assert_eq!(cave.scan_radius_chunks, 4);
    assert_eq!(layers.select(0, -1).band, Some(-1));
    assert_eq!(layers.select(0, -1).camera_height, -1);
}

#[test]
fn the_band_holds_until_the_feet_are_past_the_margin() {
    let mut layers = selector();
    layers.toggle();
    assert_eq!(layers.select(0, 40).band, Some(2));
    assert_eq!(layers.select(0, 49).band, Some(2));
    assert_eq!(layers.select(0, 30).band, Some(2));
    assert_eq!(layers.select(0, 50).band, Some(3));
    assert_eq!(layers.select(0, 46).band, Some(3));
    assert_eq!(layers.select(0, 45).band, Some(2));
}

#[test]
fn turning_it_off_or_a_new_world_goes_back_to_the_surface() {
    let mut layers = selector();
    layers.toggle();
    layers.select(0, 40);
    assert!(!layers.toggle());
    assert_eq!(layers.select(0, 40).id, 0);
    layers.toggle();
    layers.reset();
    assert!(!layers.is_cave());
    assert_eq!(layers.select(0, 40).band, None);
}
