use std::collections::HashMap;
use std::path::{Path, PathBuf};

use minimap_core::config::TerrainConfig;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::errors::CaptureError;
use minimap_runtime::services::CaptureReader;
use minimap_runtime::structs::{CaptureEvent, SubmitStatus};

use crate::services::BenchTerrain;
use crate::structs::BenchInput;

// the chunk sets benches run against: the synthetic terrain, then every capture in fixtures/
pub struct BenchInputs;

impl BenchInputs {
    pub const SYNTHETIC_RADIUS: i32 = 24;

    pub fn fixtures_dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("fixtures")
    }

    pub fn synthetic() -> BenchInput {
        BenchInput {
            name: "synthetic".to_string(),
            terrain: TerrainConfig::default(),
            chunks: BenchTerrain::chunks(ChunkPos::new(0, 0, 0), Self::SYNTHETIC_RADIUS),
        }
    }

    pub fn all() -> Vec<BenchInput> {
        let mut inputs = vec![Self::synthetic()];
        for path in Self::captures(&Self::fixtures_dir()) {
            match Self::from_capture(&path) {
                Ok(input) if !input.chunks.is_empty() => inputs.push(input),
                Ok(_) => {}
                Err(error) => eprintln!("skipping {}: {error}", path.display()),
            }
        }
        inputs
    }

    pub fn captures(dir: &Path) -> Vec<PathBuf> {
        let mut out: Vec<PathBuf> = std::fs::read_dir(dir)
            .map(|entries| {
                entries.flatten().map(|e| e.path()).filter(|p| p.extension().is_some_and(|e| e == "lmcap")).collect()
            })
            .unwrap_or_default();
        out.sort();
        out
    }

    // the last loaded surface of every chunk the capture probed, merged as the chunk store merges them
    pub fn from_capture(path: &Path) -> Result<BenchInput, CaptureError> {
        let mut reader = CaptureReader::open(path)?;
        let terrain = reader.header().terrain.clone();
        let mut latest: HashMap<ChunkPos, ChunkSurface> = HashMap::new();
        while let Some(event) = reader.next_event()? {
            if let CaptureEvent::Submit { pos, status: SubmitStatus::Loaded, probe } = event {
                let stored = latest.get(&pos).copied().unwrap_or_else(ChunkSurface::empty);
                let merged = stored.merge(&probe.surface).surface;
                if merged.has_base_data() {
                    latest.insert(pos, merged);
                }
            }
        }
        let mut chunks: Vec<(ChunkPos, ChunkSurface)> = latest.into_iter().collect();
        chunks.sort_by_key(|(pos, _)| (pos.dim, pos.z, pos.x));
        let name = path.file_stem().map_or_else(|| "capture".to_string(), |s| s.to_string_lossy().into_owned());
        Ok(BenchInput { name, terrain, chunks })
    }
}
