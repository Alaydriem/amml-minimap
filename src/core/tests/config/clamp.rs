use minimap_core::config::{Config, ConfigClamp};

#[test]
fn defaults_need_no_clamping() {
    assert_eq!(ConfigClamp::apply(Config::default()), Config::default());
}

#[test]
fn max_radius_below_min_is_raised_to_min() {
    let mut input = Config::default();
    input.mini_map.min_radius_chunks = 4;
    input.mini_map.max_radius_chunks = 3;
    input.mini_map.radius_chunks = 1;

    let config = ConfigClamp::apply(input);

    assert_eq!(config.mini_map.max_radius_chunks, 4);
    assert_eq!(config.mini_map.radius_chunks, 4);
}

#[test]
fn radius_is_held_between_its_limits() {
    let mut input = Config::default();
    input.mini_map.radius_chunks = 60;

    assert_eq!(ConfigClamp::apply(input).mini_map.radius_chunks, 16);
}

#[test]
fn max_zoom_below_min_is_raised_to_min() {
    let mut input = Config::default();
    input.world_map.min_zoom = 5.0;
    input.world_map.max_zoom = 2.0;

    assert_eq!(ConfigClamp::apply(input).world_map.max_zoom, 5.0);
}

#[test]
fn band_margin_is_kept_under_half_the_band() {
    let mut input = Config::default();
    input.cave_mode.band_size = 8;
    input.cave_mode.band_margin = 6;

    assert_eq!(ConfigClamp::apply(input).cave_mode.band_margin, 3);
}

#[test]
fn more_than_64_block_colours_are_cut_to_64() {
    let mut input = Config::default();
    input.terrain.block_colors = (0..70).map(|n| (format!("minecraft:block_{n:02}"), [n as u8, 0, 0, 255])).collect();
    input.terrain.block_colors.insert(String::new(), [0, 0, 0, 255]);

    let config = ConfigClamp::apply(input);

    assert_eq!(config.terrain.block_colors.len(), 64);
    assert!(!config.terrain.block_colors.contains_key(""));
}
