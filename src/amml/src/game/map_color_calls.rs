use std::ffi::c_void;

use amml_game::{Block, EntryId, GameEntries, RawObject};

use crate::types::{BlockPos, MapColorComponent, MceColor, MinimapEntry};

// the game calls behind a vanilla map colour. Every call is for the game thread; each returns `None` when an
// entry is off or the game gives null
pub struct MapColorCalls<'a> {
    entries: &'a GameEntries,
}

impl<'a> MapColorCalls<'a> {
    pub fn new(entries: &'a GameEntries) -> Self {
        Self { entries }
    }

    fn function(&self, id: EntryId) -> Option<*const c_void> {
        let address = usize::try_from(self.entries.get(id)?).ok()?;
        (address != 0).then_some(address as *const c_void)
    }

    /// the BlockSource of the dimension's main chunk source, which the colour samplers read biomes through
    ///
    /// # Safety
    /// `dimension` must be a live Dimension of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn main_source(&self, dimension: *mut c_void) -> Option<*mut c_void> {
        let slot = self.entries.get(MinimapEntry::DIMENSION_GET_BLOCK_SOURCE_FROM_MAIN_CHUNK_SOURCE)?;
        // SAFETY: the caller guarantees a live Dimension, whose first qword is its vtable; the entry is the slot
        let function = unsafe { RawObject::slot(dimension, slot) }?;
        // SAFETY: the slot holds `BlockSource& Dimension::getBlockSourceFromMainChunkSource() const`
        unsafe { RawObject::call_pointer(function, dimension) }
    }

    /// the block's BlockMapColorComponent: its colour and tint method; `None` when the block has none
    ///
    /// # Safety
    /// `block` must be a live Block of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn component(&self, block: *mut c_void) -> Option<MapColorComponent> {
        let function = self.function(MinimapEntry::BLOCK_TRY_GET_MAP_COLOR_COMPONENT)?;
        // SAFETY: the entry is `BlockMapColorComponent const* Block::tryGetComponent<BlockMapColorComponent>()
        // const`, a pointer or null in rax, and the caller guarantees a live block
        let component = unsafe { RawObject::call_pointer(function, block) }?;
        // SAFETY: a non-null pointer is a live BlockMapColorComponent, which MapColorComponent mirrors
        unsafe { RawObject::read::<MapColorComponent>(component, 0) }
    }

    /// the tint the biome policy of `tint` gives at `pos`
    ///
    /// # Safety
    /// `source` must be a live BlockSource of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn policy_tint(&self, tint: u8, source: *mut c_void, pos: &BlockPos) -> Option<MceColor> {
        let get_policy = self.function(MinimapEntry::BIOME_COLOR_SAMPLING_GET_MAP_POLICY)?;
        let slot = self.entries.get(MinimapEntry::MAP_POLICY_GET)?;
        // SAFETY: the entry is the static `MapPolicy const& BiomeColorSampling::getMapPolicy(TintMethod)`: the
        // byte in cl, the reference in rax
        let call: unsafe extern "C" fn(u8) -> *mut c_void = unsafe { std::mem::transmute(get_policy) };
        // SAFETY: the signature is as above; getMapPolicy takes any tint method
        let policy = unsafe { call(tint) };
        if policy.is_null() {
            return None;
        }
        // SAFETY: the game returns a live policy, whose first qword is its vtable; the entry is the `get` slot
        let get = unsafe { RawObject::slot(policy, slot) }?;
        // SAFETY: the slot holds `mce::Color MapPolicy::get(BlockSource const&, BlockPos const&) const`, a member
        // that returns the 16-byte colour through a hidden pointer: this, out, source, pos
        let get: unsafe extern "C" fn(*mut c_void, *mut MceColor, *mut c_void, *const BlockPos) -> *mut MceColor =
            unsafe { std::mem::transmute(get) };
        let mut out = MceColor::default();
        // SAFETY: the signature is as above; `out`, `source` and `pos` live across the call
        unsafe { get(policy, &raw mut out, source, pos) };
        Some(out)
    }

    /// the block type's own map colour, for a block with no map colour component
    ///
    /// # Safety
    /// `block` and `source` must be a live Block and BlockSource of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn block_type_color(&self, block: *mut c_void, source: *mut c_void, pos: &BlockPos) -> Option<MceColor> {
        let slot = self.entries.get(MinimapEntry::BLOCK_TYPE_GET_MAP_COLOR)?;
        // SAFETY: the caller guarantees a live Block of the game build the entries were resolved for
        let block_type = unsafe { Block::from_raw(block, self.entries) }?.block_type_ptr()?;
        // SAFETY: a non-null block type's first qword is its vtable; the entry is the `getMapColor` slot
        let function = unsafe { RawObject::slot(block_type, slot) }?;
        // SAFETY: the slot holds `mce::Color BlockType::getMapColor(BlockSource const&, BlockPos const&, Block
        // const&) const`: this, out, source, pos, then the block on the stack
        let call: unsafe extern "C" fn(
            *mut c_void,
            *mut MceColor,
            *mut c_void,
            *const BlockPos,
            *mut c_void,
        ) -> *mut MceColor = unsafe { std::mem::transmute(function) };
        let mut out = MceColor::default();
        // SAFETY: the signature is as above; every argument lives across the call
        unsafe { call(block_type, &raw mut out, source, pos, block) };
        Some(out)
    }

    // calls the static `int f(Biome const&, BlockPos const&)` at `id`
    #[allow(unsafe_code)]
    unsafe fn biome_sampler(&self, id: EntryId, biome: *mut c_void, pos: &BlockPos) -> Option<u32> {
        let function = self.function(id)?;
        // SAFETY: the callers name a static sampler of that signature, which returns the colour int in eax
        let call: unsafe extern "C" fn(*mut c_void, *const BlockPos) -> u32 = unsafe { std::mem::transmute(function) };
        // SAFETY: the caller guarantees a live biome; `pos` lives across the call
        Some(unsafe { call(biome, pos) })
    }

    /// the water colour of `biome` as 0xAARRGGBB
    ///
    /// # Safety
    /// `biome` must be a live Biome of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn water_color(&self, biome: *mut c_void, pos: &BlockPos) -> Option<u32> {
        // SAFETY: forwarded to the caller; the entry is `BiomeColorSampling::getWaterColor`
        unsafe { self.biome_sampler(MinimapEntry::BIOME_COLOR_SAMPLING_GET_WATER_COLOR, biome, pos) }
    }

    /// the map grass colour of `biome` as 0x00RRGGBB
    ///
    /// # Safety
    /// `biome` must be a live Biome of the game build the entries were resolved for
    #[allow(unsafe_code)]
    pub unsafe fn grass_color(&self, biome: *mut c_void, pos: &BlockPos) -> Option<u32> {
        // SAFETY: forwarded to the caller; the entry is `BiomeColorSampling::getMapGrassColor`
        unsafe { self.biome_sampler(MinimapEntry::BIOME_COLOR_SAMPLING_GET_MAP_GRASS_COLOR, biome, pos) }
    }
}
