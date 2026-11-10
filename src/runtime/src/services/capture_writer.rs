use std::fs::File;
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::Receiver;

use crate::errors::CaptureError;
use crate::services::{CaptureCodec, CaptureSink};
use crate::structs::{CaptureEvent, CaptureHeader, CaptureReport};

// writes recorded events to a .lmcap file on its own thread, behind a bounded queue
pub struct CaptureWriter {
    sink: CaptureSink,
    state: Arc<(AtomicBool, AtomicU64)>,
    thread: Option<JoinHandle<Result<u64, CaptureError>>>,
}

impl CaptureWriter {
    pub const CAPACITY: usize = 4096;
    const LEVEL: i32 = 3;
    const POLL: Duration = Duration::from_millis(20);

    pub fn start(path: &Path, header: &CaptureHeader) -> Result<Self, CaptureError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = BufWriter::new(File::create(path)?);
        CaptureCodec::write_header(&mut file, header, CaptureCodec::ZSTD)?;
        let encoder = zstd::Encoder::new(file, Self::LEVEL)?;
        let (tx, rx) = crossbeam_channel::bounded(Self::CAPACITY);
        let state = Arc::new((AtomicBool::new(false), AtomicU64::new(0)));
        let thread_state = Arc::clone(&state);
        let thread = std::thread::Builder::new()
            .name("minimap-capture".into())
            .spawn(move || Self::run(&rx, &thread_state, encoder))?;
        Ok(Self { sink: CaptureSink::new(tx, Arc::clone(&state)), state, thread: Some(thread) })
    }

    pub fn sink(&self) -> CaptureSink {
        self.sink.clone()
    }

    // stops recording, writes what is queued, and closes the file
    pub fn finish(mut self) -> CaptureReport {
        self.state.0.store(true, Ordering::Release);
        let dropped = self.state.1.load(Ordering::Relaxed);
        match self.thread.take().map(JoinHandle::join) {
            Some(Ok(Ok(written))) => CaptureReport { written, dropped, finished: true },
            Some(Ok(Err(error))) => {
                curia::warn!("capture not finished", { error: error.to_string() });
                CaptureReport { written: 0, dropped, finished: false }
            }
            _ => CaptureReport { written: 0, dropped, finished: false },
        }
    }

    fn run(
        events: &Receiver<CaptureEvent>,
        state: &(AtomicBool, AtomicU64),
        mut encoder: zstd::Encoder<'static, BufWriter<File>>,
    ) -> Result<u64, CaptureError> {
        let mut scratch = Vec::with_capacity(4096);
        let mut written = 0u64;
        loop {
            let closed = state.0.load(Ordering::Acquire);
            match events.recv_timeout(Self::POLL) {
                Ok(event) => {
                    scratch.clear();
                    CaptureCodec::encode(&event, &mut scratch);
                    encoder.write_all(&scratch)?;
                    written += 1;
                }
                Err(_) if closed => break,
                Err(_) => {}
            }
        }
        encoder.finish()?.flush()?;
        Ok(written)
    }
}

impl Drop for CaptureWriter {
    fn drop(&mut self) {
        self.state.0.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
