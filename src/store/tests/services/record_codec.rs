use minimap_core::structs::pos::ChunkPos;
use minimap_store::errors::StoreError;
use minimap_store::services::RecordCodec;
use minimap_store::structs::JournalRecord;

use crate::support::TestRecords;

#[test]
fn the_header_layout_is_a_format_contract() {
    let record = TestRecords::record(ChunkPos::new(-5, 9, 2), 70, 1);
    let bytes = RecordCodec::encode(&record, &mut Vec::new()).unwrap();
    assert_eq!(&bytes[0..4], b"LMJ2");
    assert_eq!(&bytes[4..8], &(-5i32).to_le_bytes());
    assert_eq!(&bytes[8..12], &9i32.to_le_bytes());
    assert_eq!(&bytes[12..16], &2i32.to_le_bytes());
    assert_eq!(&bytes[16..24], &7u64.to_le_bytes());
    assert_eq!(&bytes[24..32], &0u64.to_le_bytes());
    assert_eq!(bytes[32], RecordCodec::HAS_PATCH);
    assert_eq!(&bytes[33..37], &(3264u32 + 1024).to_le_bytes());
}

#[test]
fn a_record_without_a_patch_decodes_with_no_patch() {
    let record = JournalRecord::new(ChunkPos::new(1, 1, 0), 3, TestRecords::surface(64), None);
    let bytes = RecordCodec::encode(&record, &mut Vec::new()).unwrap();
    assert_eq!(bytes[32], 0);
    let decoded = RecordCodec::decode(&bytes).unwrap();
    assert_eq!(decoded.patch, None);
    assert_eq!(decoded, record);
}

#[test]
fn short_and_foreign_records_are_rejected() {
    let bytes = RecordCodec::encode(&TestRecords::record(ChunkPos::new(0, 0, 0), 64, 1), &mut Vec::new()).unwrap();
    assert!(matches!(RecordCodec::decode(&bytes[..10]), Err(StoreError::Truncated(10))));
    let mut foreign = bytes;
    foreign[1] = b'X';
    assert!(matches!(RecordCodec::decode(&foreign), Err(StoreError::BadMagic)));
}

#[test]
fn a_record_keeps_the_table_id_its_palette_used() {
    let mut record = TestRecords::record(ChunkPos::new(0, 0, 0), 70, 1);
    record.table_id = 99;
    let decoded = RecordCodec::decode(&RecordCodec::encode(&record, &mut Vec::new()).unwrap()).unwrap();
    assert_eq!(decoded.table_id, 99);
}

#[test]
fn a_journal_record_in_the_older_format_is_rejected() {
    let record = TestRecords::record(ChunkPos::new(0, 0, 0), 70, 1);
    let mut bytes = RecordCodec::encode(&record, &mut Vec::new()).unwrap();
    bytes[0..4].copy_from_slice(b"LMJ1");
    assert!(matches!(RecordCodec::decode(&bytes), Err(StoreError::BadMagic)));
}
