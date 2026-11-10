use amm_abi::types::AmmStatus;
use amm_core::errors::LinkError;
use amm_core::structs::manager::TextureGrant as ManagerGrant;
use minimap_core::structs::draw::TextureGrant;
use minimap_link::services::TextureGrantReader;

#[test]
fn a_grant_is_taken_as_the_manager_gives_it() {
    let grant = TextureGrantReader::read(Ok(ManagerGrant { bytes: 512 * 1024 * 1024, count: 2200 }));

    assert_eq!(grant, TextureGrant { bytes: 512 * 1024 * 1024, count: 2200 });
}

#[test]
fn unsupported_texture_budget_means_the_old_manager_grant() {
    let grant = TextureGrantReader::read(Err(LinkError::Status(AmmStatus::Unsupported as i32)));

    assert_eq!(grant, TextureGrant::OLD_MANAGER);
}

#[test]
fn any_other_error_also_falls_back_to_the_old_manager_grant() {
    let grant = TextureGrantReader::read(Err(LinkError::Status(AmmStatus::UnknownHandle as i32)));

    assert_eq!(grant, TextureGrant::OLD_MANAGER);
}
