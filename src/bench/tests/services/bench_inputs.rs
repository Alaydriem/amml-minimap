use minimap_bench::services::{BenchInputs, BenchTerrain, SyntheticCapture};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::services::CaptureReader;
use minimap_runtime::structs::CaptureEvent;

#[test]
fn the_synthetic_capture_reads_back_as_bench_chunks() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("walk.lmcap");
    let report = SyntheticCapture::write(&path).unwrap();
    assert!(report.finished);
    assert_eq!(report.dropped, 0);
    let input = BenchInputs::from_capture(&path).unwrap();
    assert_eq!(input.name, "walk");
    assert!(input.chunks.len() > 300, "{}", input.chunks.len());
    let (pos, surface) = &input.chunks[0];
    assert_eq!(*surface, BenchTerrain::stored(*pos));
    assert!(input.chunks.iter().any(|(p, _)| p.dim == 1), "the walk ends in the nether");
}

#[test]
fn the_synthetic_capture_opens_the_world_map_and_leaves_at_the_end() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("walk.lmcap");
    SyntheticCapture::write(&path).unwrap();
    let mut reader = CaptureReader::open(&path).unwrap();
    let mut events = Vec::new();
    while let Some(event) = reader.next_event().unwrap() {
        events.push(event);
    }
    assert_eq!(events.first(), Some(&CaptureEvent::Enter { dim: 0 }));
    assert_eq!(events.last(), Some(&CaptureEvent::Leave));
    let ticks = events.iter().filter(|e| matches!(e, CaptureEvent::Tick(_))).count();
    assert_eq!(ticks, SyntheticCapture::TICKS as usize);
    assert_eq!(events.iter().filter(|e| matches!(e, CaptureEvent::Keybind(_))).count(), 2);
}

#[test]
fn captures_are_found_in_a_folder_and_other_files_ignored() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("b.lmcap"), b"").unwrap();
    std::fs::write(dir.path().join("a.lmcap"), b"").unwrap();
    std::fs::write(dir.path().join("notes.txt"), b"").unwrap();
    let names: Vec<String> = BenchInputs::captures(dir.path())
        .iter()
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .collect();
    assert_eq!(names, vec!["a.lmcap", "b.lmcap"]);
}

#[test]
fn the_synthetic_input_is_centred_on_the_origin() {
    let input = BenchInputs::synthetic();
    let side = 2 * BenchInputs::SYNTHETIC_RADIUS + 1;
    assert_eq!(input.chunks.len(), (side * side) as usize);
    assert!(input.chunks.iter().any(|(p, _)| *p == ChunkPos::new(0, 0, 0)));
}
