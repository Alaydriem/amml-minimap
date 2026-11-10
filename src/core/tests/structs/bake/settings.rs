use minimap_core::config::TerrainConfig;
use minimap_core::structs::bake::{BakeSettings, RenderStyle};

#[test]
fn settings_hash_of_the_defaults_is_stable_across_versions() {
    let settings = BakeSettings::new(&TerrainConfig::default());
    assert_eq!(settings.settings_hash(), GOLDEN_DEFAULT_HASH);
}

#[test]
fn every_bake_setting_changes_the_hash() {
    let base = BakeSettings::new(&TerrainConfig::default());
    let variants = [
        BakeSettings { style: RenderStyle::Gradient, ..base },
        BakeSettings { shadow_level: 131, ..base },
        BakeSettings { scale: 3, ..base },
        BakeSettings { pcf_radius: 2, ..base },
        BakeSettings { light_azimuth: 300.0, ..base },
        BakeSettings { light_zenith: 45.0, ..base },
        BakeSettings { transparent_water: false, ..base },
        BakeSettings { water_opacity_per_depth: 0.2, ..base },
        BakeSettings { water_max_opacity: 0.5, ..base },
    ];
    for variant in variants {
        assert_ne!(variant.settings_hash(), base.settings_hash(), "{variant:?}");
    }
}

#[test]
fn shadow_rays_stop_at_each_dimension_top() {
    assert_eq!(BakeSettings::dimension_top(0), 320);
    assert_eq!(BakeSettings::dimension_top(1), 128);
    assert_eq!(BakeSettings::dimension_top(2), 256);
    assert_eq!(BakeSettings::dimension_top(7), 0);
}

const GOLDEN_DEFAULT_HASH: u64 = 16_646_131_953_660_463_746;
