use std::cell::Cell;
use std::path::{Path, PathBuf};

use amml_api::{ConnectionKind, WorldInfo};
use minimap_os::services::ProcessProbe;
use minimap_store::structs::WorldIdentity;

// names the world the player entered, which keys its cache. The connection is read first: a remote connection is a
// server, whatever storage folder is seen. A local world is cached under `<world>/@alaydriem/minimap/data`. A server
// is named by host, port and the low 32 bits of the seed; a world with neither is `local/S<seed>` under the data
// folder. Without a seed (the loader gives 0 when the game has none) such a world cannot be told from another, so it
// gets a folder of its own for this entry only, `local/unseeded-<process>-<n>`, which a later start removes once
// that process has ended
pub struct WorldIdentityReader {
    data_dir: PathBuf,
    // world entries keyed without a seed by this reader
    unseeded: Cell<u32>,
}

impl WorldIdentityReader {
    pub const NAMESPACE_FOLDER: &'static str = "@alaydriem";
    pub const MOD_FOLDER: &'static str = "minimap";
    pub const DATA_FOLDER: &'static str = "data";
    pub const LOCAL: &'static str = "local";
    pub const UNSEEDED_PREFIX: &'static str = "unseeded-";

    pub fn new(data_dir: &Path) -> Self {
        Self { data_dir: data_dir.to_path_buf(), unseeded: Cell::new(0) }
    }

    // `info`: the loader's facts of the joined world, `None` when it gives none
    pub fn read(&self, info: Option<&WorldInfo>) -> WorldIdentity {
        let seed = info.and_then(|info| info.seed).filter(|seed| *seed != 0);
        let remote = info
            .and_then(|info| info.connection.as_ref())
            .filter(|connection| connection.kind == ConnectionKind::Remote && !connection.host.is_empty());
        if let Some(connection) = remote {
            return WorldIdentity::Remote {
                host: connection.host.clone(),
                port: connection.port,
                seed: Self::low_bits(seed.unwrap_or(0)),
            };
        }
        let folder = info.and_then(|info| info.storage_folder.as_ref()).filter(|folder| !folder.as_os_str().is_empty());
        if let Some(folder) = folder {
            return WorldIdentity::Local {
                world_dir: folder.join(Self::NAMESPACE_FOLDER).join(Self::MOD_FOLDER).join(Self::DATA_FOLDER),
            };
        }
        let name = match seed {
            Some(seed) => format!("S{}", Self::low_bits(seed)),
            None => {
                let entry = self.unseeded.get();
                self.unseeded.set(entry.wrapping_add(1));
                format!("{}{}-{entry}", Self::UNSEEDED_PREFIX, std::process::id())
            }
        };
        WorldIdentity::Local { world_dir: self.data_dir.join(Self::LOCAL).join(name) }
    }

    // whether `identity` keeps its cache in the world's own folder
    pub fn from_world_folder(&self, identity: &WorldIdentity) -> bool {
        match identity {
            WorldIdentity::Local { world_dir } => !world_dir.starts_with(self.data_dir.join(Self::LOCAL)),
            WorldIdentity::Remote { .. } => false,
        }
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

    // the cache names keep the 32-bit seed the game gave before the 64-bit seed was read
    fn low_bits(seed: u64) -> u32 {
        (seed & u64::from(u32::MAX)) as u32
    }

    // the process id in a folder name `unseeded-<pid>-<n>`
    fn unseeded_pid(name: &str) -> Option<u32> {
        let (pid, entry) = name.strip_prefix(Self::UNSEEDED_PREFIX)?.split_once('-')?;
        entry.parse::<u32>().ok()?;
        pid.parse().ok()
    }
}
