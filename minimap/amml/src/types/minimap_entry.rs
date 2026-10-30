use amml_game::EntryId;

// the entries only minimap reads; the shared ones are amml_game::EntryId constants
pub struct MinimapEntry;

impl MinimapEntry {
    pub const CLIENT_INSTANCE_GET_GAME_CONNECTION_INFO: EntryId = EntryId::new("ClientInstance.getGameConnectionInfo");
    // sizeof(GameConnectionInfo): the engaged flag of a std::optional<GameConnectionInfo> sits right after it
    pub const GAME_CONNECTION_INFO_SIZE: EntryId = EntryId::new("GameConnectionInfo.size");
    // the non-deleting destructor, `(this) -> void`, run on minimap's own copy of the connection info
    pub const GAME_CONNECTION_INFO_DTOR: EntryId = EntryId::new("GameConnectionInfo.dtor");
    pub const LEVEL_GET_SEED: EntryId = EntryId::new("Level.getSeed");
    // the address of DBStorage's vtable: a live storage starts with it
    pub const DB_STORAGE_VTABLE: EntryId = EntryId::new("DBStorage.vtable");
    pub const DB_STORAGE_CTOR: EntryId = EntryId::new("DBStorage.ctor");
    pub const DB_STORAGE_CHECK_SHUTDOWN_DONE: EntryId = EntryId::new("DBStorage.checkShutdownDone");
    pub const LEVEL_GET_BIOME_REGISTRY: EntryId = EntryId::new("Level.getBiomeRegistry");
    pub const BIOME_REGISTRY_LOOKUP_BY_ID: EntryId = EntryId::new("BiomeRegistry.lookupById");
    pub const LEVEL_CHUNK_GET_BIOME: EntryId = EntryId::new("LevelChunk.getBiome");
    // a 9-slot listener table to copy; its own slots 0, 2 and 4 must never be used
    pub const BLOCK_SOURCE_LISTENER_VTABLE: EntryId = EntryId::new("BlockSourceListener.vtable");
    pub const BLOCK_SOURCE_LISTENER_ON_SOURCE_DESTROYED: EntryId =
        EntryId::new("BlockSourceListener.onSourceDestroyed");
    pub const BLOCK_SOURCE_LISTENER_ON_BLOCK_CHANGED: EntryId = EntryId::new("BlockSourceListener.onBlockChanged");
    pub const BLOCK_SOURCE_ADD_LISTENER: EntryId = EntryId::new("BlockSource.addListener");
    pub const BLOCK_SOURCE_REMOVE_LISTENER: EntryId = EntryId::new("BlockSource.removeListener");
    pub const BLOCK_SOURCE_GET_DIMENSION_ID: EntryId = EntryId::new("BlockSource.getDimensionId");
    pub const DIMENSION_GET_BLOCK_SOURCE_FROM_MAIN_CHUNK_SOURCE: EntryId =
        EntryId::new("Dimension.getBlockSourceFromMainChunkSource");
    pub const BLOCK_TRY_GET_MAP_COLOR_COMPONENT: EntryId = EntryId::new("Block.tryGetComponent.BlockMapColorComponent");
    pub const BIOME_COLOR_SAMPLING_GET_MAP_POLICY: EntryId = EntryId::new("BiomeColorSampling.getMapPolicy");
    pub const MAP_POLICY_GET: EntryId = EntryId::new("MapPolicy.get");
    pub const BLOCK_TYPE_GET_MAP_COLOR: EntryId = EntryId::new("BlockType.getMapColor");
    pub const BIOME_COLOR_SAMPLING_GET_WATER_COLOR: EntryId = EntryId::new("BiomeColorSampling.getWaterColor");
    pub const BIOME_COLOR_SAMPLING_GET_MAP_GRASS_COLOR: EntryId = EntryId::new("BiomeColorSampling.getMapGrassColor");
}
