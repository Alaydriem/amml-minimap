use std::path::Path;

use minimap_core::config::TerrainConfig;
use minimap_core::structs::chunk::{ChunkProbe, ChunkSurface};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::services::{CaptureReader, CaptureWriter};
use minimap_runtime::structs::{CaptureEvent, CaptureHeader, SubmitStatus};

fn events(count: i32) -> Vec<CaptureEvent> {
    (0..count)
        .map(|x| CaptureEvent::Submit {
            pos: ChunkPos::new(x, 0, 0),
            status: SubmitStatus::Loaded,
            probe: Box::new(ChunkProbe::new(ChunkSurface::empty(), [0; 256])),
        })
        .collect()
}

fn read_all(path: &Path) -> Vec<CaptureEvent> {
    let mut reader = CaptureReader::open(path).unwrap();
    let mut out = Vec::new();
    while let Some(event) = reader.next_event().unwrap() {
        out.push(event);
    }
    out
}

fn write(path: &Path, events: &[CaptureEvent]) {
    let writer = CaptureWriter::start(path, &CaptureHeader::new(&TerrainConfig::default())).unwrap();
    let sink = writer.sink();
    for event in events {
        while !sink.record(event.clone()) {
            std::thread::yield_now();
        }
    }
    let report = writer.finish();
    assert!(report.finished);
    assert_eq!(report.written, events.len() as u64);
}

#[test]
fn recorded_events_read_back_in_order() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("captures").join("walk.lmcap");
    let recorded = events(50);
    write(&path, &recorded);
    let mut reader = CaptureReader::open(&path).unwrap();
    assert_eq!(reader.header().terrain, TerrainConfig::default());
    assert_eq!(reader.next_event().unwrap(), Some(recorded[0].clone()));
    assert_eq!(read_all(&path), recorded);
}

#[test]
fn a_finished_writer_drops_further_events_instead_of_blocking() {
    let dir = tempfile::tempdir().unwrap();
    let writer =
        CaptureWriter::start(&dir.path().join("a.lmcap"), &CaptureHeader::new(&TerrainConfig::default())).unwrap();
    let sink = writer.sink();
    writer.finish();
    assert!(!sink.record(CaptureEvent::Leave));
}

#[test]
fn an_unpacked_capture_reads_the_same_and_a_cut_tail_ends_at_the_last_whole_record() {
    let dir = tempfile::tempdir().unwrap();
    let packed = dir.path().join("walk.lmcap");
    let raw = dir.path().join("walk.lmraw");
    let recorded = events(10);
    write(&packed, &recorded);
    assert_eq!(CaptureReader::open(&packed).unwrap().unpack_to(&raw).unwrap(), 10);
    assert_eq!(read_all(&raw), recorded);
    let bytes = std::fs::read(&raw).unwrap();
    std::fs::write(&raw, &bytes[..bytes.len() - 100]).unwrap();
    assert_eq!(read_all(&raw), recorded[..9].to_vec());
}
