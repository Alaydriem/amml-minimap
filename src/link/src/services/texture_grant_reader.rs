use amm_abi::types::AmmStatus;
use amm_core::errors::LinkError;
use amm_core::structs::manager::TextureGrant as ManagerGrant;
use minimap_core::structs::draw::TextureGrant;

// the manager's answer to texture_budget as minimap's grant; a manager from before grants answers Unsupported
pub struct TextureGrantReader;

impl TextureGrantReader {
    pub fn read(answer: Result<ManagerGrant, LinkError>) -> TextureGrant {
        match answer {
            Ok(grant) => TextureGrant { bytes: grant.bytes, count: grant.count },
            Err(LinkError::Status(status)) if status == AmmStatus::Unsupported as i32 => TextureGrant::OLD_MANAGER,
            Err(error) => {
                curia::warn!("texture budget not read; using the old manager grant", { error: error.to_string() });
                TextureGrant::OLD_MANAGER
            }
        }
    }
}
