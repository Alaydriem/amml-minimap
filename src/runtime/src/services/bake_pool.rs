use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender};
use minimap_core::structs::bake::BakeJob;

use crate::errors::RuntimeError;
use crate::services::bake_worker::BakeWorker;
use crate::structs::{BakeResult, Shared};

// bake threads fed by a bounded job queue; results come back on a bounded queue
pub struct BakePool {
    jobs: Option<Sender<BakeJob>>,
    results: Receiver<BakeResult>,
    workers: Vec<JoinHandle<()>>,
    stop: Arc<AtomicBool>,
}

impl BakePool {
    pub const RESULT_CAPACITY: usize = 16;

    // threads 0 means a quarter of the cores, at least one
    pub fn new(threads: usize, capacity: usize, shared: Arc<Shared>) -> Result<Self, RuntimeError> {
        let threads = if threads == 0 { Self::default_threads() } else { threads };
        let (job_tx, job_rx) = crossbeam_channel::bounded(capacity.max(1));
        let (result_tx, result_rx) = crossbeam_channel::bounded(Self::RESULT_CAPACITY);
        let stop = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(threads);
        for i in 0..threads {
            let worker = BakeWorker::new(Arc::clone(&shared), job_rx.clone(), result_tx.clone(), Arc::clone(&stop));
            workers.push(std::thread::Builder::new().name(format!("minimap-bake-{i}")).spawn(move || worker.run())?);
        }
        Ok(Self { jobs: Some(job_tx), results: result_rx, workers, stop })
    }

    pub fn default_threads() -> usize {
        std::thread::available_parallelism().map_or(1, |n| (n.get() / 4).max(1))
    }

    // never blocks; false when the queue is full
    pub fn try_submit(&self, job: BakeJob) -> bool {
        self.jobs.as_ref().is_some_and(|jobs| jobs.try_send(job).is_ok())
    }

    pub fn queued(&self) -> usize {
        self.jobs.as_ref().map_or(0, Sender::len)
    }

    pub fn drain(&self, out: &mut Vec<BakeResult>, max: usize) {
        while out.len() < max {
            match self.results.try_recv() {
                Ok(result) => out.push(result),
                Err(_) => break,
            }
        }
    }

    // lets the bakes already running complete and hands their results to `apply`; queued jobs are
    // dropped (their chunks stay journal-dirty); gives up after `timeout`
    pub fn finish(&mut self, timeout: Duration, mut apply: impl FnMut(BakeResult)) {
        self.stop.store(true, Ordering::SeqCst);
        self.jobs = None;
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            match self.results.recv_timeout(Duration::from_millis(10)) {
                Ok(result) => apply(result),
                Err(_) if self.workers.iter().all(|w| w.is_finished()) => break,
                Err(_) => {}
            }
        }
        while let Ok(result) = self.results.try_recv() {
            apply(result);
        }
        self.shutdown(Duration::from_secs(1));
    }

    // stops taking jobs, then waits up to `timeout` for the threads to finish their current job
    pub fn shutdown(&mut self, timeout: Duration) {
        self.stop.store(true, Ordering::SeqCst);
        self.jobs = None;
        let deadline = Instant::now() + timeout;
        while self.workers.iter().any(|w| !w.is_finished()) && Instant::now() < deadline {
            while self.results.try_recv().is_ok() {}
            std::thread::sleep(Duration::from_millis(5));
        }
        for worker in self.workers.drain(..) {
            if worker.is_finished() {
                let _ = worker.join();
            }
        }
    }
}

impl Drop for BakePool {
    fn drop(&mut self) {
        self.shutdown(Duration::from_secs(2));
    }
}
