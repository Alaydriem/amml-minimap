use minimap_core::structs::biome::BiomeKey;
use minimap_core::structs::pos::{ChunkPos, RegionPos};
use minimap_store::errors::StoreError;
use minimap_store::services::RegionCodec;
use minimap_store::structs::RegionSnapshot;

use crate::support::TestRecords;

fn snapshot() -> RegionSnapshot {
    let mut snapshot = RegionSnapshot::new(RegionPos::new(-2, 5, 1), 0x0102_0304_0506_0708);
    snapshot.apply(&TestRecords::record(ChunkPos::new(-32, 80, 1), 70, 0xFF00_00AA));
    snapshot.settings_hash = 0x0102_0304_0506_0708;
    snapshot
}

#[test]
fn the_header_layout_is_a_format_contract() {
    let bytes = RegionCodec::encode(&snapshot()).unwrap();
    assert_eq!(&bytes[0..4], b"LMRG");
    assert_eq!(&bytes[4..6], &2u16.to_le_bytes());
    assert_eq!(&bytes[8..12], &1i32.to_le_bytes());
    assert_eq!(&bytes[12..16], &(-2i32).to_le_bytes());
    assert_eq!(&bytes[16..20], &5i32.to_le_bytes());
    assert_eq!(&bytes[20..28], &0x0102_0304_0506_0708u64.to_le_bytes());
    assert_eq!(&bytes[28..36], &1u64.to_le_bytes());
    let tile_len = u32::from_le_bytes(bytes[60..64].try_into().unwrap()) as usize;
    let surface_len = u32::from_le_bytes(bytes[64..68].try_into().unwrap()) as usize;
    assert_eq!(bytes.len(), RegionCodec::HEADER_BYTES + tile_len + surface_len);
    assert_eq!(RegionCodec::HEADER_BYTES, 88);
    assert_eq!(&bytes[76..84], &0u64.to_le_bytes());
}

#[test]
fn decoding_restores_surfaces_tile_and_hash() {
    let original = snapshot();
    let decoded = RegionCodec::decode(&RegionCodec::encode(&original).unwrap()).unwrap();
    assert_eq!(decoded, original);
}

#[test]
fn a_flipped_byte_in_a_section_fails_its_checksum() {
    let mut bytes = RegionCodec::encode(&snapshot()).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0xFF;
    assert!(matches!(RegionCodec::decode(&bytes), Err(StoreError::ChecksumMismatch("surface"))));
    let mut bytes = RegionCodec::encode(&snapshot()).unwrap();
    bytes[RegionCodec::HEADER_BYTES] ^= 0xFF;
    assert!(matches!(RegionCodec::decode(&bytes), Err(StoreError::ChecksumMismatch("tile"))));
}

#[test]
fn short_foreign_and_future_files_are_rejected() {
    let bytes = RegionCodec::encode(&snapshot()).unwrap();
    assert!(matches!(RegionCodec::decode(&bytes[..40]), Err(StoreError::Truncated(40))));
    assert!(matches!(RegionCodec::decode(&bytes[..bytes.len() - 1]), Err(StoreError::Truncated(_))));
    let mut foreign = bytes.clone();
    foreign[0] = b'X';
    assert!(matches!(RegionCodec::decode(&foreign), Err(StoreError::BadMagic)));
    let mut future = bytes;
    future[4] = 3;
    assert!(matches!(RegionCodec::decode(&future), Err(StoreError::UnsupportedVersion(3))));
}

fn snapshot_with_biome(table_id: u64) -> RegionSnapshot {
    let mut snapshot = RegionSnapshot::new(RegionPos::new(0, 0, 0), 1);
    snapshot.table_id = table_id;
    let mut surface = TestRecords::surface(70);
    surface.biome.set(0, Some(BiomeKey(3)));
    snapshot.surfaces[4 * 16 + 3] = Some(Box::new(surface));
    snapshot
}

#[test]
fn a_region_file_keeps_each_chunks_biome_palette_and_its_table_id() {
    let bytes = RegionCodec::encode(&snapshot_with_biome(7)).unwrap();
    assert_eq!(&bytes[76..84], &7u64.to_le_bytes());
    let decoded = RegionCodec::decode(&bytes).unwrap();
    assert_eq!(decoded.table_id, 7);
    assert_eq!(decoded.surfaces[4 * 16 + 3].as_ref().unwrap().biome.key(0), Some(BiomeKey(3)));
    let biomes = RegionCodec::decode_biomes(&bytes, 7).unwrap();
    assert_eq!(biomes.key((3, 4), 0), Some(BiomeKey(3)));
    assert_eq!(biomes.key((0, 0), 0), None);
}

#[test]
fn palettes_keyed_against_another_table_read_as_unknown() {
    let bytes = RegionCodec::encode(&snapshot_with_biome(7)).unwrap();
    assert_eq!(RegionCodec::decode_biomes(&bytes, 8).unwrap().key((3, 4), 0), None);
    let mut decoded = RegionCodec::decode(&bytes).unwrap();
    decoded.retain_biomes(8);
    let surface = decoded.surfaces[4 * 16 + 3].as_ref().unwrap();
    assert_eq!(surface.biome.key(0), None);
    assert_eq!(surface.height[0], 70);
    assert_eq!(decoded.table_id, 8);
}

#[test]
fn a_version_one_region_file_is_rejected() {
    let mut bytes = RegionCodec::encode(&snapshot()).unwrap();
    bytes[4..6].copy_from_slice(&1u16.to_le_bytes());
    assert!(matches!(RegionCodec::decode(&bytes), Err(StoreError::UnsupportedVersion(1))));
    assert!(matches!(RegionCodec::decode_biomes(&bytes, 0), Err(StoreError::UnsupportedVersion(1))));
}

#[test]
fn an_unknown_table_drops_no_palettes() {
    let mut decoded = RegionCodec::decode(&RegionCodec::encode(&snapshot_with_biome(7)).unwrap()).unwrap();
    decoded.retain_biomes(0);
    assert_eq!(decoded.surfaces[4 * 16 + 3].as_ref().unwrap().biome.key(0), Some(BiomeKey(3)));
    assert_eq!(decoded.table_id, 7);
}
