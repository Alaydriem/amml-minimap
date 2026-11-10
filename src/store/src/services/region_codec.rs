use minimap_core::structs::biome::RegionBiomes;
use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;
use xxhash_rust::xxh3::xxh3_64;

use crate::errors::StoreError;
use crate::structs::RegionSnapshot;

// the `.lmr` region snapshot format: an 88-byte header, then two zstd sections
pub struct RegionCodec;

impl RegionCodec {
    pub const MAGIC: &'static [u8; 4] = b"LMRG";
    pub const VERSION: u16 = 2;
    pub const HEADER_BYTES: usize = 88;
    const LEVEL: i32 = 3;
    const SURFACE_BYTES: usize = std::mem::size_of::<ChunkSurface>();

    pub fn encode(snapshot: &RegionSnapshot) -> Result<Vec<u8>, StoreError> {
        let tile = zstd::bulk::compress(&snapshot.tile, Self::LEVEL)?;
        let mut raw_surfaces = Vec::with_capacity(snapshot.surface_count() * Self::SURFACE_BYTES);
        for surface in snapshot.surfaces.iter().flatten() {
            raw_surfaces.extend_from_slice(bytemuck::bytes_of(surface.as_ref()));
        }
        let surfaces = zstd::bulk::compress(&raw_surfaces, Self::LEVEL)?;
        let mut out = Vec::with_capacity(Self::HEADER_BYTES + tile.len() + surfaces.len());
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&Self::VERSION.to_le_bytes());
        out.extend_from_slice(&0u16.to_le_bytes());
        out.extend_from_slice(&snapshot.region.dim.to_le_bytes());
        out.extend_from_slice(&snapshot.region.x.to_le_bytes());
        out.extend_from_slice(&snapshot.region.z.to_le_bytes());
        out.extend_from_slice(&snapshot.settings_hash.to_le_bytes());
        for word in snapshot.present_bits() {
            out.extend_from_slice(&word.to_le_bytes());
        }
        out.extend_from_slice(&(tile.len() as u32).to_le_bytes());
        out.extend_from_slice(&(surfaces.len() as u32).to_le_bytes());
        out.extend_from_slice(&(xxh3_64(&tile) as u32).to_le_bytes());
        out.extend_from_slice(&(xxh3_64(&surfaces) as u32).to_le_bytes());
        out.extend_from_slice(&snapshot.table_id.to_le_bytes());
        out.extend_from_slice(&0u32.to_le_bytes());
        out.extend_from_slice(&tile);
        out.extend_from_slice(&surfaces);
        Ok(out)
    }

    pub fn decode(bytes: &[u8]) -> Result<RegionSnapshot, StoreError> {
        Self::check_header(bytes)?;
        let region = RegionPos::new(Self::i32_at(bytes, 12), Self::i32_at(bytes, 16), Self::i32_at(bytes, 8));
        let settings_hash = Self::u64_at(bytes, 20);
        let present =
            [Self::u64_at(bytes, 28), Self::u64_at(bytes, 36), Self::u64_at(bytes, 44), Self::u64_at(bytes, 52)];
        let tile_len = Self::u32_at(bytes, 60) as usize;
        let surface_len = Self::u32_at(bytes, 64) as usize;
        if bytes.len() < Self::HEADER_BYTES + tile_len + surface_len {
            return Err(StoreError::Truncated(bytes.len()));
        }
        let tile_bytes = &bytes[Self::HEADER_BYTES..Self::HEADER_BYTES + tile_len];
        let surface_bytes = &bytes[Self::HEADER_BYTES + tile_len..Self::HEADER_BYTES + tile_len + surface_len];
        if xxh3_64(tile_bytes) as u32 != Self::u32_at(bytes, 68) {
            return Err(StoreError::ChecksumMismatch("tile"));
        }
        if xxh3_64(surface_bytes) as u32 != Self::u32_at(bytes, 72) {
            return Err(StoreError::ChecksumMismatch("surface"));
        }
        let mut snapshot = RegionSnapshot::new(region, settings_hash);
        snapshot.table_id = Self::u64_at(bytes, 76);
        let tile = zstd::bulk::decompress(tile_bytes, Tile::BYTES)?;
        if tile.len() != Tile::BYTES {
            return Err(StoreError::WrongSize { section: "tile", expected: Tile::BYTES, actual: tile.len() });
        }
        snapshot.tile = tile.into_boxed_slice();
        let count: usize = present.iter().map(|w| w.count_ones() as usize).sum();
        let expected = count * Self::SURFACE_BYTES;
        let raw = zstd::bulk::decompress(surface_bytes, expected.max(1))?;
        if raw.len() != expected {
            return Err(StoreError::WrongSize { section: "surface", expected, actual: raw.len() });
        }
        let mut chunks = raw.as_chunks::<{ Self::SURFACE_BYTES }>().0.iter();
        for i in 0..256 {
            if (present[i / 64] >> (i % 64)) & 1 == 0 {
                continue;
            }
            if let Some(chunk) = chunks.next() {
                snapshot.surfaces[i] = Some(Box::new(bytemuck::pod_read_unaligned(chunk.as_slice())));
            }
        }
        Ok(snapshot)
    }

    // the biome palettes only: the tile section is skipped without decompressing it. Palettes keyed
    // against another table than `expected_table_id` read as no biomes
    pub fn decode_biomes(bytes: &[u8], expected_table_id: u64) -> Result<RegionBiomes, StoreError> {
        Self::check_header(bytes)?;
        let present =
            [Self::u64_at(bytes, 28), Self::u64_at(bytes, 36), Self::u64_at(bytes, 44), Self::u64_at(bytes, 52)];
        let tile_len = Self::u32_at(bytes, 60) as usize;
        let surface_len = Self::u32_at(bytes, 64) as usize;
        if bytes.len() < Self::HEADER_BYTES + tile_len + surface_len {
            return Err(StoreError::Truncated(bytes.len()));
        }
        let surface_bytes = &bytes[Self::HEADER_BYTES + tile_len..Self::HEADER_BYTES + tile_len + surface_len];
        if xxh3_64(surface_bytes) as u32 != Self::u32_at(bytes, 72) {
            return Err(StoreError::ChecksumMismatch("surface"));
        }
        let mut biomes = RegionBiomes::empty();
        if Self::u64_at(bytes, 76) != expected_table_id {
            return Ok(biomes);
        }
        let count: usize = present.iter().map(|w| w.count_ones() as usize).sum();
        let expected = count * Self::SURFACE_BYTES;
        let raw = zstd::bulk::decompress(surface_bytes, expected.max(1))?;
        if raw.len() != expected {
            return Err(StoreError::WrongSize { section: "surface", expected, actual: raw.len() });
        }
        let mut chunks = raw.as_chunks::<{ Self::SURFACE_BYTES }>().0.iter();
        for i in 0..256 {
            if (present[i / 64] >> (i % 64)) & 1 == 0 {
                continue;
            }
            if let Some(chunk) = chunks.next() {
                let surface: ChunkSurface = bytemuck::pod_read_unaligned(chunk.as_slice());
                biomes.set((i % 16, i / 16), surface.biome);
            }
        }
        Ok(biomes)
    }

    fn check_header(bytes: &[u8]) -> Result<(), StoreError> {
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
        Ok(())
    }

    fn u32_at(bytes: &[u8], at: usize) -> u32 {
        u32::from_le_bytes([bytes[at], bytes[at + 1], bytes[at + 2], bytes[at + 3]])
    }

    fn i32_at(bytes: &[u8], at: usize) -> i32 {
        Self::u32_at(bytes, at) as i32
    }

    fn u64_at(bytes: &[u8], at: usize) -> u64 {
        u64::from(Self::u32_at(bytes, at)) | (u64::from(Self::u32_at(bytes, at + 4)) << 32)
    }
}
