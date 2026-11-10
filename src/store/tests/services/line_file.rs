use minimap_store::services::LineFile;

#[test]
fn lines_are_appended_and_truncate_starts_again() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("logs").join("telemetry.csv");
    LineFile::append(&path, "a,b", true).unwrap();
    LineFile::append(&path, "1,2", false).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "a,b\n1,2\n");
    LineFile::append(&path, "a,b", true).unwrap();
    assert_eq!(std::fs::read_to_string(&path).unwrap(), "a,b\n");
}
