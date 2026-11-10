use minimap_store::errors::StoreError;
use minimap_store::services::BiomeKeyFile;

#[test]
fn names_are_read_back_in_key_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("biomes.lmb");
    let names = vec!["plains".to_string(), "forest".to_string(), "hell".to_string()];
    BiomeKeyFile::save(&path, 42, &names).unwrap();
    assert_eq!(BiomeKeyFile::load(&path).unwrap(), Some((42, names)));
}

#[test]
fn a_missing_file_is_no_table() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(BiomeKeyFile::load(&dir.path().join("biomes.lmb")).unwrap(), None);
}

#[test]
fn an_unreadable_file_is_set_aside_and_reads_as_no_table() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("biomes.lmb");
    std::fs::write(&path, b"junk").unwrap();
    assert_eq!(BiomeKeyFile::load(&path).unwrap(), None);
    assert!(dir.path().join("biomes.lmb.corrupt").exists());
    assert!(!path.exists());
}

#[test]
fn an_overlong_name_is_refused() {
    assert!(BiomeKeyFile::encode(1, &["x".repeat(65)]).is_err());
}

#[test]
fn a_count_above_the_table_capacity_is_refused_before_any_allocation() {
    let mut bytes = BiomeKeyFile::encode(1, &["plains".to_string()]).unwrap();
    bytes[16..20].copy_from_slice(&u32::MAX.to_le_bytes());
    assert!(matches!(BiomeKeyFile::decode(&bytes), Err(StoreError::TooManyBiomes(_))));
}
