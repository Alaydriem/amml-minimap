use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;

use crate::errors::StoreError;
use crate::structs::JournalRecord;

// journal record: a 37-byte header, then zstd(surface ++ optional 16 x 16 patch)
pub struct RecordCodec;

impl RecordCodec {
    pub const MAGIC: &'static [u8; 4] = b"LMJ2";
    pub const HEADER_BYTES: usize = 37;
    pub const HAS_PATCH: u8 = 1;
    const LEVEL: i32 = 3;
    const SURFACE_BYTES: usize = std::mem::size_of::<ChunkSurface>();
    const PATCH_BYTES: usize = 256 * 4;

    pub fn encode(record: &JournalRecord, scratch: &mut Vec<u8>) -> Result<Vec<u8>, StoreError> {
        scratch.clear();
        scratch.extend_from_slice(bytemuck::bytes_of(&record.surface));
        if let Some(patch) = &record.patch {
            scratch.extend_from_slice(bytemuck::cast_slice(patch.as_slice()));
        }
        let payload = zstd::bulk::compress(scratch, Self::LEVEL)?;
        let mut out = Vec::with_capacity(Self::HEADER_BYTES + payload.len());
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&record.pos.x.to_le_bytes());
        out.extend_from_slice(&record.pos.z.to_le_bytes());
        out.extend_from_slice(&record.pos.dim.to_le_bytes());
        out.extend_from_slice(&record.settings_hash.to_le_bytes());
        out.extend_from_slice(&record.table_id.to_le_bytes());
        out.push(if record.patch.is_some() { Self::HAS_PATCH } else { 0 });
        out.extend_from_slice(&(scratch.len() as u32).to_le_bytes());
        out.extend_from_slice(&payload);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<JournalRecord, StoreError> {
        if bytes.len() < Self::HEADER_BYTES {
            return Err(StoreError::Truncated(bytes.len()));
        }
        if &bytes[0..4] != Self::MAGIC {
            return Err(StoreError::BadMagic);
        }
        let int = |at: usize| i32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]]);
        let pos = ChunkPos::new(int(4), int(8), int(12));
        let long = |at: usize| u64::from(int(at) as u32) | (u64::from(int(at + 4) as u32) << 32);
        let settings_hash = long(16);
        let table_id = long(24);
        let has_patch = bytes[32] & Self::HAS_PATCH != 0;
        let expected = Self::SURFACE_BYTES + if has_patch { Self::PATCH_BYTES } else { 0 };
        let raw = zstd::bulk::decompress(&bytes[Self::HEADER_BYTES..], expected)?;
        if raw.len() != expected {
            return Err(StoreError::WrongSize { section: "record", expected, actual: raw.len() });
        }
        let surface: ChunkSurface = bytemuck::pod_read_unaligned(&raw[..Self::SURFACE_BYTES]);
        let patch = has_patch.then(|| {
            let mut patch = [0u32; 256];
            for (i, word) in raw[Self::SURFACE_BYTES..].as_chunks::<4>().0.iter().enumerate() {
                patch[i] = u32::from_le_bytes(*word);
            }
            patch
        });
        let mut record = JournalRecord::new(pos, settings_hash, surface, patch);
        record.table_id = table_id;
        Ok(record)
    }
}
