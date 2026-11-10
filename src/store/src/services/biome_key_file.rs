use std::fs;
use std::io::Write;
use std::path::Path;

use minimap_core::services::biome::{BiomeIdNames, BiomeKeyTable};

use crate::errors::StoreError;

// `biomes.lmb`: a world's biome key table. A 20-byte header (magic, version, reserved, table ID,
// count), then zstd of each name as [length u8][utf-8 bytes] in key order
pub struct BiomeKeyFile;

impl BiomeKeyFile {
    pub const MAGIC: &'static [u8; 4] = b"LMBK";
    pub const VERSION: u16 = 1;
    pub const HEADER_BYTES: usize = 20;
    const LEVEL: i32 = 3;

    pub fn encode(table_id: u64, names: &[String]) -> Result<Vec<u8>, StoreError> {
        let mut raw = Vec::with_capacity(names.len() * 16);
        for name in names {
            if name.is_empty() || name.len() > BiomeIdNames::MAX_NAME_BYTES {
                return Err(StoreError::BadBiomeName(BiomeIdNames::MAX_NAME_BYTES));
            }
            raw.push(name.len() as u8);
            raw.extend_from_slice(name.as_bytes());
        }
        let payload = zstd::bulk::compress(&raw, Self::LEVEL)?;
        let mut out = Vec::with_capacity(Self::HEADER_BYTES + payload.len());
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&Self::VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&table_id.to_le_bytes());
        out.extend_from_slice(&(names.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<(u64, Vec<String>), StoreError> {
        if bytes.len() < Self::HEADER_BYTES {
            return Err(StoreError::Truncated(bytes.len()));
        }
        if &bytes[0..4] != Self::MAGIC {
            return Err(StoreError::BadMagic);
        }
        let version = u16::from_le_bytes([bytes[4], bytes[5]]);
        if version != Self::VERSION {
            return Err(StoreError::UnsupportedVersion(version));
        }
        let mut id = [0u8; 8];
        id.copy_from_slice(&bytes[8..16]);
        let count = u32::from_le_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]) as usize;
        // a damaged count must not size the buffers below
        if count > BiomeKeyTable::CAPACITY {
            return Err(StoreError::TooManyBiomes(count));
        }
        let max = count * (BiomeIdNames::MAX_NAME_BYTES + 1);
        let raw = zstd::bulk::decompress(&bytes[Self::HEADER_BYTES..], max.max(1))?;
        let mut names = Vec::with_capacity(count);
        let mut at = 0;
        while names.len() < count {
            let len = usize::from(*raw.get(at).ok_or(StoreError::Truncated(raw.len()))?);
            let name = raw.get(at + 1..at + 1 + len).ok_or(StoreError::Truncated(raw.len()))?;
            let name = std::str::from_utf8(name).map_err(|_| StoreError::BadBiomeName(BiomeIdNames::MAX_NAME_BYTES))?;
            names.push(name.to_string());
            at += 1 + len;
        }
        Ok((u64::from_le_bytes(id), names))
    }

    // None when the file is missing; an unreadable file is renamed to .corrupt and also gives None
    pub fn load(path: &Path) -> Result<Option<(u64, Vec<String>)>, StoreError> {
        let bytes = match fs::read(path) {
            Ok(bytes) => bytes,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.into()),
        };
        match Self::decode(&bytes) {
            Ok(table) => Ok(Some(table)),
            Err(error) => {
                curia::warn!("biome key file is unreadable; it is renamed to .corrupt", { path: path.display().to_string(), error: error.to_string() });
                if let Err(error) = fs::rename(path, path.with_extension("lmb.corrupt")) {
                    curia::warn!("could not rename a corrupt file", { path: path.display().to_string(), error: error.to_string() });
                }
                Ok(None)
            }
        }
    }

    // writes beside the target, flushes, then renames over it
    pub fn save(path: &Path, table_id: u64, names: &[String]) -> Result<(), StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = Self::encode(table_id, names)?;
        let tmp = path.with_extension("lmb.tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(&bytes)?;
            file.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }
}
