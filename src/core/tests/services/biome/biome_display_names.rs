use minimap_core::services::biome::BiomeDisplayNames;

#[test]
fn internal_names_read_as_player_names() {
    assert_eq!(BiomeDisplayNames::display("hell"), Some("Nether Wastes"));
    assert_eq!(BiomeDisplayNames::display("roofed_forest"), Some("Dark Forest"));
    assert_eq!(BiomeDisplayNames::display("extreme_hills"), Some("Windswept Hills"));
    assert_eq!(BiomeDisplayNames::display("mesa"), Some("Badlands"));
    assert_eq!(BiomeDisplayNames::display("plains"), Some("Plains"));
    assert_eq!(label("forest"), "Forest");
}

#[test]
fn every_entry_is_found_by_its_internal_name() {
    for (internal, display) in BiomeDisplayNames::ENTRIES {
        assert_eq!(BiomeDisplayNames::display(internal), Some(*display));
    }
}

fn label(internal: &str) -> String {
    let mut out = String::new();
    BiomeDisplayNames::write_label(internal, &mut out);
    out
}

#[test]
fn the_game_s_namespaced_names_find_their_player_names() {
    assert_eq!(label("minecraft:plains"), "Plains");
    assert_eq!(label("minecraft:roofed_forest"), "Dark Forest");
    assert_eq!(BiomeDisplayNames::display("my_pack:plains"), None, "another namespace is not vanilla");
}

#[test]
fn a_name_the_table_does_not_know_is_shown_in_words() {
    assert_eq!(label("my_pack:glow_marsh"), "Glow Marsh");
    assert_eq!(label("minecraft:pale_garden_2"), "Pale Garden 2");
    assert_eq!(label("lonely"), "Lonely");
}
