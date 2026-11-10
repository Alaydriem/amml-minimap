use minimap_core::services::draw::TextureBudget;
use minimap_core::structs::draw::TextureGrant;

const MIB: u64 = 1024 * 1024;

#[test]
fn budget_is_the_setting_when_the_grant_is_larger() {
    let grant = TextureGrant { bytes: 1024 * MIB, count: 4096 };

    assert_eq!(TextureBudget::tiles(grant, 496), 1984);
}

#[test]
fn budget_is_the_grant_less_headroom_when_the_setting_is_larger() {
    let grant = TextureGrant { bytes: 256 * MIB, count: 4096 };

    assert_eq!(TextureBudget::tiles(grant, 496), 960);
}

#[test]
fn the_grant_count_caps_the_tiles() {
    let grant = TextureGrant { bytes: 1024 * MIB, count: 500 };

    assert_eq!(TextureBudget::tiles(grant, 496), 484);
}

#[test]
fn an_old_manager_gives_192_tiles() {
    assert_eq!(TextureBudget::tiles(TextureGrant::OLD_MANAGER, 496), 192);
}

#[test]
fn a_grant_inside_the_headroom_still_gives_one_tile() {
    let grant = TextureGrant { bytes: 8 * MIB, count: 8 };

    assert_eq!(TextureBudget::tiles(grant, 496), 1);
}
