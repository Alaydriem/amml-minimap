use crate::services::draw::GpuResidency;
use crate::structs::draw::TextureGrant;

// how many region tiles fit on the GPU: the player's setting, capped by the manager's grant less room for the
// corner map, the font atlas and egui's own textures
pub struct TextureBudget;

impl TextureBudget {
    const HEADROOM_BYTES: u64 = 16 * 1024 * 1024;
    const HEADROOM_TEXTURES: u32 = 16;
    const MIB: u64 = 1024 * 1024;

    pub fn tiles(grant: TextureGrant, setting_mb: i32) -> usize {
        let setting = u64::try_from(setting_mb.max(1)).unwrap_or(1) * Self::MIB;
        let granted = grant.bytes.saturating_sub(Self::HEADROOM_BYTES);
        let by_bytes = setting.min(granted) / GpuResidency::TILE_BYTES as u64;
        let by_count = u64::from(grant.count.saturating_sub(Self::HEADROOM_TEXTURES));
        usize::try_from(by_bytes.min(by_count).max(1)).unwrap_or(1)
    }
}
