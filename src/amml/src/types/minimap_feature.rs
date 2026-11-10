use amml_game::{EntryId, GameEntries};

use crate::types::MinimapEntry;

// the optional features; each is off when one of its entries does not resolve
pub struct MinimapFeature;

impl MinimapFeature {
    // the server address and the world seed that name a server's cache
    pub const SERVER_IDENTITY: &'static str = "minimap.server_identity";
    // the local world's folder, where the LeviLamina build kept its cache
    pub const WORLD_FOLDER: &'static str = "minimap.world_folder";
    pub const BIOMES: &'static str = "minimap.biomes";
    pub const BLOCK_CHANGES: &'static str = "minimap.block_changes";
    pub const VANILLA_COLORS: &'static str = "minimap.vanilla_colors";

    pub const ALL: [&'static str; 5] =
        [Self::SERVER_IDENTITY, Self::WORLD_FOLDER, Self::BIOMES, Self::BLOCK_CHANGES, Self::VANILLA_COLORS];

    const SERVER_IDENTITY_ENTRIES: [EntryId; 3] = [
        MinimapEntry::CLIENT_INSTANCE_GET_GAME_CONNECTION_INFO,
        MinimapEntry::GAME_CONNECTION_INFO_SIZE,
        MinimapEntry::LEVEL_GET_SEED,
    ];
    const WORLD_FOLDER_ENTRIES: [EntryId; 3] =
        [MinimapEntry::DB_STORAGE_VTABLE, MinimapEntry::DB_STORAGE_CTOR, MinimapEntry::DB_STORAGE_CHECK_SHUTDOWN_DONE];
    const BIOMES_ENTRIES: [EntryId; 3] = [
        MinimapEntry::LEVEL_GET_BIOME_REGISTRY,
        MinimapEntry::BIOME_REGISTRY_LOOKUP_BY_ID,
        MinimapEntry::LEVEL_CHUNK_GET_BIOME,
    ];
    const BLOCK_CHANGES_ENTRIES: [EntryId; 6] = [
        MinimapEntry::BLOCK_SOURCE_LISTENER_VTABLE,
        MinimapEntry::BLOCK_SOURCE_LISTENER_ON_SOURCE_DESTROYED,
        MinimapEntry::BLOCK_SOURCE_LISTENER_ON_BLOCK_CHANGED,
        MinimapEntry::BLOCK_SOURCE_ADD_LISTENER,
        MinimapEntry::BLOCK_SOURCE_REMOVE_LISTENER,
        MinimapEntry::BLOCK_SOURCE_GET_DIMENSION_ID,
    ];
    const VANILLA_COLORS_ENTRIES: [EntryId; 7] = [
        MinimapEntry::DIMENSION_GET_BLOCK_SOURCE_FROM_MAIN_CHUNK_SOURCE,
        MinimapEntry::BLOCK_TRY_GET_MAP_COLOR_COMPONENT,
        MinimapEntry::BIOME_COLOR_SAMPLING_GET_MAP_POLICY,
        MinimapEntry::MAP_POLICY_GET,
        MinimapEntry::BLOCK_TYPE_GET_MAP_COLOR,
        MinimapEntry::BIOME_COLOR_SAMPLING_GET_WATER_COLOR,
        MinimapEntry::BIOME_COLOR_SAMPLING_GET_MAP_GRASS_COLOR,
    ];

    // the entries of `feature`; empty for a name that is not a feature
    pub fn entries(feature: &str) -> &'static [EntryId] {
        match feature {
            Self::SERVER_IDENTITY => &Self::SERVER_IDENTITY_ENTRIES,
            Self::WORLD_FOLDER => &Self::WORLD_FOLDER_ENTRIES,
            Self::BIOMES => &Self::BIOMES_ENTRIES,
            Self::BLOCK_CHANGES => &Self::BLOCK_CHANGES_ENTRIES,
            Self::VANILLA_COLORS => &Self::VANILLA_COLORS_ENTRIES,
            _ => &[],
        }
    }

    // every entry of `feature` resolved
    pub fn is_on(entries: &GameEntries, feature: &str) -> bool {
        let ids = Self::entries(feature);
        !ids.is_empty() && ids.iter().all(|id| entries.get(*id).is_some())
    }
}
