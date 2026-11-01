use std::path::{Path, PathBuf};

use minimap_core::structs::pos::RegionPos;

use crate::services::WaypointFile;
use crate::structs::WorldIdentity;

pub struct CachePaths;

impl CachePaths {
    pub const FOLDER: &'static str = "minimap_cache";
    // the folder name of caches written before the rename; `adopt_legacy` moves such a folder to `FOLDER`
    pub const LEGACY_FOLDER: &'static str = "levi_minimap_cache";

    pub fn root(identity: &WorldIdentity, mod_data_dir: &Path) -> PathBuf {
        match identity {
            WorldIdentity::Local { world_dir } => world_dir.join(Self::FOLDER),
            WorldIdentity::Remote { host, port, seed } => mod_data_dir
                .join("servers")
                .join(format!("{}_{}_S{}", Self::sanitize(host), port, seed))
                .join(Self::FOLDER),
        }
    }

    // beside the cache folder, so it is kept with the world even when the disk cache is off
    pub fn waypoints_file(identity: &WorldIdentity, mod_data_dir: &Path) -> PathBuf {
        let root = Self::root(identity, mod_data_dir);
        root.parent().unwrap_or(mod_data_dir).join(WaypointFile::NAME)
    }

    // moves a cache folder with the old name, beside `root`, to `root`, so the map a player already explored is
    // kept. Nothing moves when `root` exists or no old folder is there. Gives whether a folder moved
    pub fn adopt_legacy(root: &Path) -> std::io::Result<bool> {
        if root.exists() {
            return Ok(false);
        }
        let Some(legacy) = root.parent().map(|parent| parent.join(Self::LEGACY_FOLDER)) else { return Ok(false) };
        if !legacy.is_dir() {
            return Ok(false);
        }
        std::fs::rename(&legacy, root)?;
        Ok(true)
    }

    pub fn region_file(root: &Path, region: RegionPos) -> PathBuf {
        root.join("regions").join(region.dim.to_string()).join(format!("r.{}.{}.lmr", region.x, region.z))
    }

    pub fn biome_keys_file(root: &Path) -> PathBuf {
        root.join("biomes.lmb")
    }

    pub fn journal_dir(root: &Path, dim: i32) -> PathBuf {
        root.join("journal").join(dim.to_string())
    }

    // characters Windows does not allow in a path component become `_`
    pub fn sanitize(component: &str) -> String {
        let cleaned: String = component
            .chars()
            .map(|c| if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
            .collect();
        if cleaned.is_empty() { "unknown".to_string() } else { cleaned }
    }
}
