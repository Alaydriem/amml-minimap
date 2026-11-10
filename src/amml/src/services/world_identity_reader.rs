use std::cell::Cell;
use std::ffi::c_void;
use std::path::{Path, PathBuf};
use std::ptr::NonNull;

use amml_game::{ClientInstance, GameEntries};
use amml_msvc::MsvcHeap;
use minimap_store::structs::WorldIdentity;

use crate::game::{DbStorage, GameConnectionInfo, Level};
use crate::services::ProcessProbe;

// names the world the player entered, which keys its cache. The connection info is read first: a remote
// connection is a server, whatever storage is seen. A local world is cached under
// `<world>/@alaydriem/minimap/data`; an old LeviLamina cache is not read. A server is named by host, port and seed; a
// world with neither (the world folder feature is off, or no storage was seen) is `local/S<seed>` under the data
// folder. Without a seed such a world cannot be told from another, so it gets a folder of its own for this entry
// only, `local/unseeded-<process>-<n>`, which a later start removes once that process has ended
pub struct WorldIdentityReader {
    data_dir: PathBuf,
    heap: Option<MsvcHeap>,
    // world entries keyed without a seed in this tracker
    unseeded: Cell<u32>,
}

impl WorldIdentityReader {
    pub const NAMESPACE_FOLDER: &'static str = "@alaydriem";
    pub const MOD_FOLDER: &'static str = "minimap";
    pub const DATA_FOLDER: &'static str = "data";
    pub const LOCAL: &'static str = "local";
    pub const UNSEEDED_PREFIX: &'static str = "unseeded-";

    // `heap` frees the strings of the game's connection info; without it the server identity is not read
    pub fn new(data_dir: &Path, heap: Option<MsvcHeap>) -> Self {
        Self { data_dir: data_dir.to_path_buf(), heap, unseeded: Cell::new(0) }
    }

    // `storage` is the live DBStorage of the open local world, if any; it names the world only when the connection
    // is not remote. Game thread
    #[allow(unsafe_code)]
    pub fn read(
        &self,
        entries: &GameEntries,
        client: &ClientInstance<'_>,
        level: Option<&Level<'_>>,
        storage: Option<NonNull<c_void>>,
    ) -> WorldIdentity {
        let known_seed = level.and_then(|level| level.seed());
        let seed = known_seed.unwrap_or(0);
        // the connection comes first, as in LeviLamina's ServerInfo: a stale or foreign storage pointer must never
        // put a server's map into a local world's folder
        // SAFETY: the wrapper holds a live ClientInstance
        let connection = unsafe { GameConnectionInfo::from_raw(client.as_ptr(), entries) };
        let address = self.heap.as_ref().zip(connection).and_then(|(heap, connection)| connection.read(heap));
        if let Some(address) = address.filter(|address| address.is_remote()) {
            return WorldIdentity::Remote { host: address.host, port: address.port, seed };
        }
        // SAFETY: the caller passes the live DBStorage the storage detours hold, or nothing
        let storage = storage.and_then(|storage| unsafe { DbStorage::from_raw(storage.as_ptr(), entries) });
        if let Some(folder) = storage.and_then(|storage| storage.full_path()).filter(|folder| !folder.is_empty()) {
            return WorldIdentity::Local {
                world_dir: PathBuf::from(folder)
                    .join(Self::NAMESPACE_FOLDER)
                    .join(Self::MOD_FOLDER)
                    .join(Self::DATA_FOLDER),
            };
        }
        let folder = match known_seed {
            Some(seed) => format!("S{seed}"),
            None => {
                let entry = self.unseeded.get();
                self.unseeded.set(entry.wrapping_add(1));
                format!("{}{}-{entry}", Self::UNSEEDED_PREFIX, std::process::id())
            }
        };
        WorldIdentity::Local { world_dir: self.data_dir.join(Self::LOCAL).join(folder) }
    }

    // removes the `local/unseeded-<pid>-<n>` folders under `data_dir` whose process is no longer running. The
    // folders of this process and of any running process (another game instance) are kept, and so is a folder whose
    // name gives no process id
    pub fn sweep_unseeded(data_dir: &Path) {
        let Ok(folders) = std::fs::read_dir(data_dir.join(Self::LOCAL)) else { return };
        for folder in folders.flatten() {
            let Some(pid) = folder.file_name().to_str().and_then(Self::unseeded_pid) else { continue };
            if ProcessProbe::is_running(pid) {
                continue;
            }
            if let Err(error) = std::fs::remove_dir_all(folder.path()) {
                curia::warn!("an unseeded world cache was not removed", { error: error.to_string() });
            }
        }
    }

    // the process id in a folder name `unseeded-<pid>-<n>`
    fn unseeded_pid(name: &str) -> Option<u32> {
        let (pid, entry) = name.strip_prefix(Self::UNSEEDED_PREFIX)?.split_once('-')?;
        entry.parse::<u32>().ok()?;
        pid.parse().ok()
    }
}
