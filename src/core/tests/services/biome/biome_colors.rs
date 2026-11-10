use std::collections::BTreeMap;

use minimap_core::services::biome::BiomeColors;
use minimap_core::structs::Rgba;

fn overrides() -> BTreeMap<String, [u8; 4]> {
    BTreeMap::from([("ocean".to_string(), [0, 0, 112, 255]), ("minecraft:plains".to_string(), [1, 2, 3, 255])])
}

#[test]
fn an_override_matches_with_or_without_the_namespace() {
    let colors = BiomeColors::from_overrides(&overrides());
    assert_eq!(colors.lookup("minecraft:ocean"), Some(Rgba::new(0, 0, 112, 255)));
    assert_eq!(colors.lookup("ocean"), Some(Rgba::new(0, 0, 112, 255)));
    assert_eq!(colors.lookup("minecraft:plains"), Some(Rgba::new(1, 2, 3, 255)));
    assert_eq!(colors.lookup("plains"), Some(Rgba::new(1, 2, 3, 255)));
}

#[test]
fn a_biome_with_no_override_has_no_colour() {
    let colors = BiomeColors::from_overrides(&overrides());
    assert_eq!(colors.lookup("minecraft:desert"), None);
    assert_eq!(BiomeColors::new().lookup("ocean"), None);
}
