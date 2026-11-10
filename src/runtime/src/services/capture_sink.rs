use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crossbeam_channel::Sender;

use crate::structs::CaptureEvent;

// the cheap, cloneable end the game and render threads record into; it never blocks
#[derive(Clone)]
pub struct CaptureSink {
    events: Sender<CaptureEvent>,
    // (closed, dropped)
    state: Arc<(AtomicBool, AtomicU64)>,
}

impl CaptureSink {
    pub(crate) fn new(events: Sender<CaptureEvent>, state: Arc<(AtomicBool, AtomicU64)>) -> Self {
        Self { events, state }
    }

    // false when the event was dropped: the writer fell behind or has finished
    pub fn record(&self, event: CaptureEvent) -> bool {
        if self.state.0.load(Ordering::Acquire) {
            return false;
        }
        let sent = self.events.try_send(event).is_ok();
        if !sent {
            self.state.1.fetch_add(1, Ordering::Relaxed);
        }
        sent
    }

    pub fn dropped(&self) -> u64 {
        self.state.1.load(Ordering::Relaxed)
    }
}
