// the texture budget the mod manager grants this mod: at most `bytes` bytes in at most `count` textures
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureGrant {
    pub bytes: u64,
    pub count: u32,
}

impl TextureGrant {
    // what a manager from before texture grants gives every mod
    pub const OLD_MANAGER: TextureGrant = TextureGrant { bytes: 64 * 1024 * 1024, count: 1024 };
}
