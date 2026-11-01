use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender, TrySendError};

use crate::errors::StoreError;
use crate::services::store_worker::StoreWorker;
use crate::structs::{ShutdownReport, StoreCommand, StoreEvent, StoreSettings};

// handle to the store thread; every queue between the game and the store is bounded
pub struct StoreService {
    commands: Option<Sender<StoreCommand>>,
    events: Receiver<StoreEvent>,
    done: Receiver<()>,
    worker: Option<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
    written: Arc<AtomicU64>,
}

impl StoreService {
    // root None runs without a disk cache: appends are acknowledged, loads report missing
    pub fn start(root: Option<PathBuf>, settings: StoreSettings) -> Result<Self, StoreError> {
        let (command_tx, command_rx) = crossbeam_channel::bounded(settings.queue_capacity);
        let (event_tx, event_rx) = crossbeam_channel::bounded(settings.queue_capacity);
        let (done_tx, done_rx) = crossbeam_channel::bounded(1);
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let written = Arc::new(AtomicU64::new(0));
        let worker_written = Arc::clone(&written);
        let worker = std::thread::Builder::new().name("minimap-store".to_string()).spawn(move || {
            StoreWorker::new(root, settings, command_rx, event_tx, worker_stop, worker_written).run();
            let _ = done_tx.send(());
        })?;
        Ok(Self { commands: Some(command_tx), events: event_rx, done: done_rx, worker: Some(worker), stop, written })
    }

    // never blocks; a full queue hands the command back
    pub fn try_send(&self, command: StoreCommand) -> Result<(), StoreCommand> {
        let Some(sender) = &self.commands else { return Err(command) };
        sender.try_send(command).map_err(|error| match error {
            TrySendError::Full(command) | TrySendError::Disconnected(command) => command,
        })
    }

    // blocks while the queue is full; for worker threads, never the game or render thread
    pub fn send(&self, command: StoreCommand) -> Result<(), StoreError> {
        let Some(sender) = &self.commands else { return Err(StoreError::Stopped) };
        sender.send(command).map_err(|_| StoreError::Stopped)
    }

    // bytes the disk cache has written since the store started (journal records and region files)
    pub fn bytes_written(&self) -> u64 {
        self.written.load(Ordering::Relaxed)
    }

    pub fn drain_events(&self, out: &mut Vec<StoreEvent>, max: usize) {
        while out.len() < max {
            match self.events.try_recv() {
                Ok(event) => out.push(event),
                Err(_) => break,
            }
        }
    }

    // stops the worker after it drains, syncs and compacts; gives up waiting after `timeout`
    pub fn shutdown(&mut self, timeout: Duration) -> ShutdownReport {
        self.stop.store(true, Ordering::SeqCst);
        self.commands = None;
        let Some(worker) = self.worker.take() else { return ShutdownReport { finished: true } };
        let finished = self.done.recv_timeout(timeout).is_ok();
        if finished {
            let _ = worker.join();
        } else {
            curia::warn!("store worker did not finish in time; it is left to finish alone", { timeout_ms: timeout.as_millis() as u64 });
        }
        ShutdownReport { finished }
    }
}

impl Drop for StoreService {
    fn drop(&mut self) {
        if self.worker.is_some() {
            self.shutdown(Duration::from_secs(5));
        }
    }
}
