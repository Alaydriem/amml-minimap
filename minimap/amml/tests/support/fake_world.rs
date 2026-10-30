use std::ffi::c_void;
use std::mem::offset_of;
use std::ptr::NonNull;

use alaydriem_minimap::MinimapMod;
use alaydriem_minimap::types::MinimapEntry;
use amml_game::{EntryId, GameEntries, LevelChunkLayout2651, SubChunkLayout};

use super::{
    FakeActor, FakeChunk, FakeClient, FakeDimension, FakeLevel, FakePalette, FakeResolver, FakeSource, FakeSubChunk,
};

// a player standing in a world of one loaded chunk at (0, 0): four blocks of stone under `top`, then air
pub struct FakeWorld {
    pub palette: FakePalette,
    pub chunk: Box<FakeChunk>,
    pub source: Box<FakeSource>,
    pub dimension: Box<FakeDimension>,
    pub level: Box<FakeLevel>,
    pub actor: Box<FakeActor>,
    pub client: Box<FakeClient>,
}

static LISTENER_BASE: [usize; 9] = [0x1000, 0x1001, 0x1002, 0x1003, 0x1004, 0x1005, 0x1006, 0x1007, 0x1008];

impl FakeWorld {
    pub const SEED: u32 = 1234;
    pub const LISTENER_SOURCE_DESTROYED: u64 = 2;
    pub const LISTENER_BLOCK_CHANGED: u64 = 4;

    pub fn new(top: &str) -> Self {
        Self::with_chunk(top, 2, false)
    }

    pub fn with_chunk(top: &str, subs: usize, top_placeholder: bool) -> Self {
        let palette = FakePalette::new();
        let stone = palette.block(FakePalette::STONE);
        let stack = vec![stone, stone, stone, stone, palette.block(top)];
        let mut chunk = FakeChunk::new(0, 0, stack, palette.block(FakePalette::AIR), subs, top_placeholder);
        let mut source = FakeSource::new(chunk.ptr(), [0, 0], 0);
        let mut dimension = FakeDimension::new(0);
        let mut level = FakeLevel::new(Self::SEED);
        let mut actor = FakeActor::new([8.5, 6.62, 8.5], level.ptr(), dimension.ptr(), source.ptr());
        let client = FakeClient::new(actor.ptr(), source.ptr());
        Self { palette, chunk, source, dimension, level, actor, client }
    }

    pub fn client_ptr(&mut self) -> NonNull<c_void> {
        NonNull::new(self.client.ptr()).unwrap()
    }

    // the address of a 9-slot table of marker values, standing in for the game's BlockSourceListener table
    pub fn listener_base() -> u64 {
        LISTENER_BASE.as_ptr() as u64
    }

    pub fn listener_base_slots() -> [usize; 9] {
        LISTENER_BASE
    }

    // the value of every required entry, for the layouts of the fakes; the chunk and block material use the 26.51
    // layouts
    pub fn required() -> Vec<(EntryId, u64)> {
        let get_dimension: extern "C" fn(*mut c_void) -> *mut c_void = FakeActor::get_dimension;
        let get_source: extern "C" fn(*mut c_void) -> *mut c_void = FakeActor::get_dimension_block_source;
        let get_block: extern "C" fn(*mut c_void, *const amml_game::ChunkBlockPos) -> *mut c_void =
            FakeChunk::get_block;
        let is_placeholder: extern "C" fn(*mut c_void) -> u8 = FakeSubChunk::is_placeholder;
        vec![
            (EntryId::CLIENT_INSTANCE_UPDATE, 0x1),
            (EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER, FakeClient::GET_LOCAL_PLAYER),
            (EntryId::CLIENT_INSTANCE_IS_WORLD_ACTIVE, FakeClient::IS_WORLD_ACTIVE),
            (EntryId::CLIENT_INSTANCE_GET_MOUSE_GRABBED, FakeClient::GET_MOUSE_GRABBED),
            (EntryId::CLIENT_INSTANCE_GET_REGION, FakeClient::GET_REGION),
            (EntryId::ACTOR_STATE_VECTOR_COMPONENT, offset_of!(FakeActor, state_vector) as u64),
            (EntryId::ACTOR_ACTOR_ROTATION_COMPONENT, offset_of!(FakeActor, rotation) as u64),
            (EntryId::STATE_VECTOR_COMPONENT_POS, 0),
            (EntryId::ACTOR_ROTATION_COMPONENT_ROT, 0),
            (EntryId::ACTOR_GET_DIMENSION, get_dimension as usize as u64),
            (EntryId::DIMENSION_GET_DIMENSION_ID, FakeDimension::GET_DIMENSION_ID),
            (EntryId::ACTOR_GET_DIMENSION_BLOCK_SOURCE, get_source as usize as u64),
            (EntryId::ACTOR_IS_ALIVE, FakeActor::IS_ALIVE),
            (EntryId::ACTOR_LEVEL, offset_of!(FakeActor, level) as u64),
            (EntryId::BLOCK_SOURCE_GET_CHUNK, FakeSource::GET_CHUNK),
            (EntryId::LEVEL_CHUNK_SUB_CHUNKS, LevelChunkLayout2651::SUB_CHUNKS),
            (EntryId::BLOCK_TYPE_MATERIAL_INLINE, 0),
            (EntryId::LEVEL_CHUNK_GET_BLOCK, get_block as usize as u64),
            (EntryId::SUB_CHUNK_SIZE, size_of::<SubChunkLayout>() as u64),
            (EntryId::SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK, is_placeholder as usize as u64),
        ]
    }

    // the server identity and block changes entries the fakes support
    pub fn optional() -> Vec<(EntryId, u64)> {
        let destroy: extern "C" fn(*mut u8) = FakeClient::destroy_connection_info;
        vec![
            (MinimapEntry::GAME_CONNECTION_INFO_DTOR, destroy as usize as u64),
            (MinimapEntry::CLIENT_INSTANCE_GET_GAME_CONNECTION_INFO, FakeClient::GET_GAME_CONNECTION_INFO),
            (MinimapEntry::GAME_CONNECTION_INFO_SIZE, FakeClient::INFO_SIZE_26_51),
            (MinimapEntry::LEVEL_GET_SEED, FakeLevel::GET_SEED),
            (MinimapEntry::BLOCK_SOURCE_LISTENER_VTABLE, Self::listener_base()),
            (MinimapEntry::BLOCK_SOURCE_LISTENER_ON_SOURCE_DESTROYED, Self::LISTENER_SOURCE_DESTROYED),
            (MinimapEntry::BLOCK_SOURCE_LISTENER_ON_BLOCK_CHANGED, Self::LISTENER_BLOCK_CHANGED),
            (MinimapEntry::BLOCK_SOURCE_ADD_LISTENER, FakeSource::ADD_LISTENER),
            (MinimapEntry::BLOCK_SOURCE_REMOVE_LISTENER, FakeSource::REMOVE_LISTENER),
            (MinimapEntry::BLOCK_SOURCE_GET_DIMENSION_ID, FakeSource::GET_DIMENSION_ID),
        ]
    }

    // the minimap's entries as MinimapMod resolves them: every required one, and the optional ones the fakes
    // support when `optional`
    pub fn entries(optional: bool) -> GameEntries {
        let mut values = Self::required();
        if optional {
            values.extend(Self::optional());
        }
        MinimapMod::resolve(&FakeResolver::new(&values)).unwrap()
    }

    // every entry the fakes support, and `extra`
    pub fn entries_with(extra: &[(EntryId, u64)]) -> GameEntries {
        Self::entries_without(extra, &[])
    }

    // every entry the fakes support but `missing`, and `extra`
    pub fn entries_without(extra: &[(EntryId, u64)], missing: &[EntryId]) -> GameEntries {
        let mut values = Self::required();
        values.extend(Self::optional());
        values.extend_from_slice(extra);
        values.retain(|(id, _)| !missing.contains(id));
        MinimapMod::resolve(&FakeResolver::new(&values)).unwrap()
    }
}
