use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

// bake count and total bake time, summed by every worker, read by telemetry
#[derive(Debug, Default)]
pub struct BakeTiming {
    bakes: AtomicU64,
    nanos: AtomicU64,
}

impl BakeTiming {
    pub fn record(&self, took: Duration) {
        self.bakes.fetch_add(1, Ordering::Relaxed);
        self.nanos.fetch_add(took.as_nanos() as u64, Ordering::Relaxed);
    }

    // (bakes, nanoseconds) so far
    pub fn totals(&self) -> (u64, u64) {
        (self.bakes.load(Ordering::Relaxed), self.nanos.load(Ordering::Relaxed))
    }
}
