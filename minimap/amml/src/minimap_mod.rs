use std::ffi::c_void;
use std::sync::atomic::AtomicPtr;

use amml_game::{EntryId, EntryNeed, EntryResolver, GameEntries, GameError, HookOrder, MaterialLayout};
use amml_sdk::amml_abi::AmmlLogLevel;
use amml_sdk::{AmmlMod, Api, HookHandle, ModDirs, SdkError};

use crate::detours::{client_update, db_storage_ctor, db_storage_shutdown};
use crate::services::{BlockListener, MinimapRuntime};
use crate::types::{MinimapEntry, MinimapFeature};

// the AMML entry point: load notes the folders, enable resolves the entries, installs the hooks and starts the
// minimap, disable removes the hooks and stops it
pub struct MinimapMod;

impl MinimapMod {
    // the mod id as a static string for the host and logger; a test checks it equals AmmlMod::id()
    pub const MODULE: &'static str = "@alaydriem/minimap";
    // the entries the minimap cannot run without: the client, the player's pose, dimension and block source, and
    // the chunk reads
    pub const REQUIRED: [EntryId; 19] = [
        EntryId::CLIENT_INSTANCE_UPDATE,
        EntryId::CLIENT_INSTANCE_GET_LOCAL_PLAYER,
        EntryId::CLIENT_INSTANCE_IS_WORLD_ACTIVE,
        EntryId::CLIENT_INSTANCE_GET_MOUSE_GRABBED,
        EntryId::CLIENT_INSTANCE_GET_REGION,
        EntryId::ACTOR_STATE_VECTOR_COMPONENT,
        EntryId::ACTOR_ACTOR_ROTATION_COMPONENT,
        EntryId::STATE_VECTOR_COMPONENT_POS,
        EntryId::ACTOR_ROTATION_COMPONENT_ROT,
        EntryId::ACTOR_GET_DIMENSION,
        EntryId::DIMENSION_GET_DIMENSION_ID,
        EntryId::ACTOR_GET_DIMENSION_BLOCK_SOURCE,
        EntryId::ACTOR_IS_ALIVE,
        EntryId::ACTOR_LEVEL,
        EntryId::BLOCK_SOURCE_GET_CHUNK,
        EntryId::LEVEL_CHUNK_SUB_CHUNKS,
        EntryId::LEVEL_CHUNK_GET_BLOCK,
        EntryId::SUB_CHUNK_SIZE,
        EntryId::SUB_CHUNK_IS_PLACE_HOLDER_SUB_CHUNK,
    ];

    // every entry the minimap asks for: the required ones, then each optional feature's
    pub fn wanted() -> Vec<(EntryId, EntryNeed)> {
        let required = Self::REQUIRED.iter().map(|id| (*id, EntryNeed::Required));
        let optional = MinimapFeature::ALL.iter().flat_map(|feature| {
            MinimapFeature::entries(feature).iter().map(|id| (*id, EntryNeed::Optional { feature }))
        });
        let destructor = std::iter::once((
            MinimapEntry::GAME_CONNECTION_INFO_DTOR,
            EntryNeed::Optional { feature: MinimapFeature::SERVER_IDENTITY },
        ));
        // the block material layout: `resolve` requires it
        required.chain(optional).chain(destructor).chain(MaterialLayout::wanted()).collect()
    }

    // the entries, or an error when a required entry is missing or the block-material layout is unknown
    pub fn resolve<R: EntryResolver>(resolver: &R) -> Result<GameEntries, GameError> {
        let entries = GameEntries::resolve(resolver, &Self::wanted())?;
        MaterialLayout::of(&entries)?;
        Ok(entries)
    }

    // one line per feature that is off, naming it and the entry that turned it off. The connection info destructor
    // is asked for last under the server identity feature; when it alone is missing the feature stays on and the
    // line says what is lost
    pub fn off_lines(entries: &GameEntries) -> Vec<String> {
        let mut lines: Vec<String> = entries
            .off_features()
            .into_iter()
            .map(|(feature, cause)| {
                if feature == MinimapFeature::SERVER_IDENTITY && MinimapFeature::is_on(entries, feature) {
                    format!(
                        "{} is missing ({cause}); only the host string of each connection info copy is freed",
                        MinimapEntry::GAME_CONNECTION_INFO_DTOR.as_str()
                    )
                } else {
                    format!("{feature} is off ({cause})")
                }
            })
            .collect();
        if MinimapFeature::is_on(entries, MinimapFeature::BLOCK_CHANGES)
            && let Some((_, slots)) = BlockListener::slots(entries)
            && let Err(error) = BlockListener::check(&slots)
        {
            lines.push(format!("{} is off ({error})", MinimapFeature::BLOCK_CHANGES));
        }
        lines
    }

    #[allow(unsafe_code)]
    fn hook(
        api: &Api,
        runtime: &MinimapRuntime,
        target: u64,
        detour: *const c_void,
        original: &'static AtomicPtr<c_void>,
    ) -> Result<HookHandle, SdkError> {
        // SAFETY: `target` is the derived start of the game function each caller pairs with the detour written
        // for its signature (this in rcx, then the C++ parameters as the game passes them); each original slot
        // belongs to one hook only
        let hook = unsafe { api.hook(target, detour, HookOrder::READ_AFTER_WRITERS, original) }?;
        if runtime.keep_hook(hook) {
            return Ok(hook);
        }
        let _ = api.unhook(hook);
        Err(SdkError::Mod(format!("more than {} hooks", MinimapRuntime::HOOK_CAPACITY)))
    }

    fn remove_hooks(api: &Api, runtime: &MinimapRuntime) {
        for hook in runtime.take_hooks() {
            if let Err(error) = api.unhook(hook) {
                api.log(Self::MODULE, AmmlLogLevel::WARN, &format!("unhook failed: {error}"));
            }
        }
    }

    // the client update is required; the storage hooks serve the world folder and are skipped without it
    fn install_hooks(api: &Api, runtime: &MinimapRuntime, entries: &GameEntries) -> Result<(), SdkError> {
        let update = entries
            .get(EntryId::CLIENT_INSTANCE_UPDATE)
            .ok_or_else(|| SdkError::MissingEntry(EntryId::CLIENT_INSTANCE_UPDATE.as_str().to_string()))?;
        Self::hook(api, runtime, update, client_update::detour as *const c_void, client_update::GUARD.original())?;
        if !MinimapFeature::is_on(entries, MinimapFeature::WORLD_FOLDER) {
            return Ok(());
        }
        // both storage hooks or neither: a constructor seen without its shutdown would leave a pointer that is
        // never cleared. The shutdown hook goes first, since on its own it only clears
        let shutdown = entries.get(MinimapEntry::DB_STORAGE_CHECK_SHUTDOWN_DONE);
        let ctor = entries.get(MinimapEntry::DB_STORAGE_CTOR);
        let (Some(shutdown), Some(ctor)) = (shutdown, ctor) else { return Ok(()) };
        let installed = Self::hook(
            api,
            runtime,
            shutdown,
            db_storage_shutdown::detour as *const c_void,
            db_storage_shutdown::GUARD.original(),
        )
        .and_then(|first| {
            Self::hook(api, runtime, ctor, db_storage_ctor::detour as *const c_void, db_storage_ctor::GUARD.original())
                .inspect_err(|_| {
                    let _ = api.unhook(first);
                    runtime.forget_hook(first);
                })
        });
        if let Err(error) = installed {
            api.log(
                Self::MODULE,
                AmmlLogLevel::WARN,
                &format!("{} is off (the storage hooks: {error})", MinimapFeature::WORLD_FOLDER),
            );
        }
        Ok(())
    }
}

impl AmmlMod for MinimapMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "minimap";

    // runs under the loader lock: only the folders are noted, nothing starts a thread
    fn load(api: &Api) -> Result<(), SdkError> {
        let dirs = ModDirs::new(api, Self::MODULE)?;
        MinimapRuntime::install(&dirs.config_dir, &dirs.data_dir);
        Ok(())
    }

    fn enable(api: &Api) -> Result<(), SdkError> {
        let runtime = MinimapRuntime::get().ok_or_else(|| SdkError::Mod("load did not run".to_string()))?;
        let entries = Self::resolve(api).map_err(|error| SdkError::Mod(error.to_string()))?;
        let off = Self::off_lines(&entries);
        if off.is_empty() {
            api.log(Self::MODULE, AmmlLogLevel::INFO, "features off: none");
        }
        for line in off {
            api.log(Self::MODULE, AmmlLogLevel::WARN, &line);
        }
        if let Err(error) = Self::install_hooks(api, runtime, &entries) {
            Self::remove_hooks(api, runtime);
            return Err(error);
        }
        runtime.start(entries);
        Ok(())
    }

    // the hooks go first, so no new tick enters the minimap; then the listener leaves its source and the session
    // stops
    fn disable(api: &Api) {
        let Some(runtime) = MinimapRuntime::get() else { return };
        runtime.latch().raise();
        Self::remove_hooks(api, runtime);
        runtime.stop();
    }
}
