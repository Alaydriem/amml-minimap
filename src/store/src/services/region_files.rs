use std::fs;
use std::io::Write;
use std::path::PathBuf;

use minimap_core::structs::biome::RegionBiomes;
use minimap_core::structs::pos::RegionPos;

use crate::errors::StoreError;
use crate::services::{CachePaths, RegionCodec};
use crate::structs::RegionSnapshot;

// region snapshot files under the cache root; a bad file is set aside, never fatal
pub struct RegionFiles {
    root: PathBuf,
    // the world's biome key table; palettes read under another table are dropped
    table_id: u64,
}

impl RegionFiles {
    pub fn new(root: PathBuf) -> Self {
        Self { root, table_id: 0 }
    }

    pub fn set_table_id(&mut self, table_id: u64) {
        self.table_id = table_id;
    }

    pub fn table_id(&self) -> u64 {
        self.table_id
    }

    pub fn read(&self, region: RegionPos) -> Result<Option<RegionSnapshot>, StoreError> {
        let path = CachePaths::region_file(&self.root, region);
        let Some(bytes) = Self::bytes(&path)? else { return Ok(None) };
        match RegionCodec::decode(&bytes) {
            Ok(mut snapshot) if snapshot.region == region => {
                snapshot.retain_biomes(self.table_id);
                Ok(Some(snapshot))
            }
            Ok(_) => {
                self.set_aside(&path, "the header names another region");
                Ok(None)
            }
            Err(error) => {
                self.drop_unreadable(&path, &error);
                Ok(None)
            }
        }
    }

    // the stored biome palettes of a region, without its tile
    pub fn read_biomes(&self, region: RegionPos) -> Result<Option<RegionBiomes>, StoreError> {
        let path = CachePaths::region_file(&self.root, region);
        let Some(bytes) = Self::bytes(&path)? else { return Ok(None) };
        match RegionCodec::decode_biomes(&bytes, self.table_id) {
            Ok(biomes) => Ok(Some(biomes)),
            Err(error) => {
                self.drop_unreadable(&path, &error);
                Ok(None)
            }
        }
    }

    fn bytes(path: &std::path::Path) -> Result<Option<Vec<u8>>, StoreError> {
        match fs::read(path) {
            Ok(bytes) => Ok(Some(bytes)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    // a file of an older format is removed and rebuilt by exploring; any other bad file is set aside
    fn drop_unreadable(&self, path: &std::path::Path, error: &StoreError) {
        if !matches!(error, StoreError::UnsupportedVersion(_)) {
            self.set_aside(path, &error.to_string());
            return;
        }
        curia::info!("a region file has an older cache format; it is rebuilt as the world is explored", { path: path.display().to_string() });
        if let Err(error) = fs::remove_file(path) {
            curia::warn!("could not remove an old region file", { path: path.display().to_string(), error: error.to_string() });
        }
    }

    // the regions of one dimension that have a file; a name that is not r.<x>.<z>.lmr is skipped
    pub fn list(&self, dim: i32) -> Result<Vec<RegionPos>, StoreError> {
        let dir = CachePaths::region_file(&self.root, RegionPos::new(0, 0, dim))
            .parent()
            .map(std::path::Path::to_path_buf)
            .unwrap_or_else(|| self.root.clone());
        let entries = match fs::read_dir(&dir) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.into()),
        };
        let mut regions = Vec::new();
        for entry in entries {
            let name = entry?.file_name();
            let Some(name) = name.to_str() else { continue };
            let Some(middle) = name.strip_prefix("r.").and_then(|n| n.strip_suffix(".lmr")) else { continue };
            let mut parts = middle.split('.');
            if let (Some(x), Some(z), None) = (parts.next(), parts.next(), parts.next())
                && let (Ok(x), Ok(z)) = (x.parse(), z.parse())
            {
                regions.push(RegionPos::new(x, z, dim));
            }
        }
        regions.sort_by_key(|r| (r.z, r.x));
        Ok(regions)
    }

    // writes beside the target, flushes, then renames over it; returns the bytes written
    pub fn write(&self, snapshot: &RegionSnapshot) -> Result<usize, StoreError> {
        let path = CachePaths::region_file(&self.root, snapshot.region);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = RegionCodec::encode(snapshot)?;
        let tmp = path.with_extension("lmr.tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        fs::rename(&tmp, &path)?;
        Ok(bytes.len())
    }

    fn set_aside(&self, path: &std::path::Path, reason: &str) {
        curia::warn!("a region file is unreadable; it is renamed to .corrupt", { path: path.display().to_string(), reason: reason.to_string() });
        if let Err(error) = fs::rename(path, path.with_extension("lmr.corrupt")) {
            curia::warn!("could not rename a corrupt file", { path: path.display().to_string(), error: error.to_string() });
        }
    }
}
