use std::time::{Duration, Instant};

use minimap_core::structs::chunk::{ChunkSurface, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_store::services::StoreService;
use minimap_store::structs::{JournalRecord, StoreEvent, StoreSettings};

pub struct TestRecords;

impl TestRecords {
    // a stored chunk whose every column has the given height
    pub fn surface(height: i16) -> ChunkSurface {
        let mut probe = ChunkSurface::empty();
        for i in 0..ChunkSurface::COLUMNS {
            probe.write_column(i, ColumnSample::new(0xFF64_6464, 0, height, height, false, 0));
        }
        ChunkSurface::empty().merge(&probe).surface
    }

    pub fn record(pos: ChunkPos, height: i16, pixel: u32) -> JournalRecord {
        JournalRecord::new(pos, 7, Self::surface(height), Some([pixel; 256]))
    }

    pub fn settings() -> StoreSettings {
        StoreSettings {
            journal_max_bytes: 64 * 1024 * 1024,
            sync_interval: Duration::from_millis(100),
            queue_capacity: 64,
            idle_compaction: Duration::from_secs(3600),
        }
    }

    // polls the service until an event matches or two seconds pass
    pub fn wait_for(service: &StoreService, matches: impl Fn(&StoreEvent) -> bool) -> Option<StoreEvent> {
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut events = Vec::new();
        while Instant::now() < deadline {
            service.drain_events(&mut events, 64);
            if let Some(i) = events.iter().position(&matches) {
                return Some(events.swap_remove(i));
            }
            events.clear();
            std::thread::sleep(Duration::from_millis(10));
        }
        None
    }
}
