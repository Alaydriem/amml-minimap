use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::{SystemTime, UNIX_EPOCH};

use crossbeam_channel::{Receiver, Sender};
use minimap_core::services::export::{AreaLayout, ExportName};
use minimap_core::structs::export::ExportKind;
use minimap_core::structs::pos::RegionPos;

use crate::errors::RuntimeError;
use crate::services::export_worker::ExportWorker;
use crate::structs::{ExportJob, Shared};

// the export thread and its one-job queue; every call returns at once, results come back as messages
pub struct ExportService {
    dir: PathBuf,
    jobs: Mutex<Option<Sender<ExportJob>>>,
    results: Receiver<String>,
    busy: Arc<AtomicBool>,
    // bumped when a world is left, so a running export stops at its next row
    epoch: Arc<AtomicU64>,
    world: Mutex<Option<String>>,
    worker: Mutex<Option<JoinHandle<()>>>,
}

impl ExportService {
    // a finished export's message starts with this
    pub const EXPORTED: &'static str = "Exported ";
    pub const FOLDER: &'static str = "exports";
    const RESULT_CAPACITY: usize = 8;

    // where the files go: `export.folder` when it is set, else the user's Pictures/Screenshots folder,
    // else <mod data>/exports when there is no user profile
    pub fn folder(configured: &str, home: Option<&Path>, data_dir: &Path) -> PathBuf {
        if !configured.trim().is_empty() {
            return PathBuf::from(configured.trim());
        }
        match home {
            Some(home) => home.join("Pictures").join("Screenshots"),
            None => data_dir.join(Self::FOLDER),
        }
    }

    // the files go to `dir`, which is made on the first export
    pub fn start(dir: &Path, shared: Arc<Shared>) -> Result<Self, RuntimeError> {
        let (job_tx, job_rx) = crossbeam_channel::bounded(1);
        let (result_tx, result_rx) = crossbeam_channel::bounded(Self::RESULT_CAPACITY);
        let busy = Arc::new(AtomicBool::new(false));
        let epoch = Arc::new(AtomicU64::new(0));
        let worker = ExportWorker::new(shared, job_rx, result_tx, Arc::clone(&busy), Arc::clone(&epoch));
        let handle = std::thread::Builder::new().name("minimap-export".to_string()).spawn(move || worker.run())?;
        Ok(Self {
            dir: dir.to_path_buf(),
            jobs: Mutex::new(Some(job_tx)),
            results: result_rx,
            busy,
            epoch,
            world: Mutex::new(None),
            worker: Mutex::new(Some(handle)),
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    // `label` names the world in the file names
    pub fn enter_world(&self, label: &str) {
        *Shared::lock(&self.world) = Some(label.to_string());
    }

    pub fn leave_world(&self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        *Shared::lock(&self.world) = None;
    }

    pub fn is_busy(&self) -> bool {
        self.busy.load(Ordering::SeqCst)
    }

    // the regions on screen, composed like an area export; Err is the message to show
    pub fn export_view(
        &self,
        layout: AreaLayout,
        regions: Vec<RegionPos>,
        cache: Option<PathBuf>,
        dim: i32,
    ) -> Result<(), String> {
        let path = self.path(dim, ExportKind::View)?;
        let epoch = self.epoch.load(Ordering::SeqCst);
        self.submit(ExportJob::Area { path, layout, regions, cache, dim, epoch })
    }

    pub fn export_area(
        &self,
        layout: AreaLayout,
        regions: Vec<RegionPos>,
        cache: Option<PathBuf>,
        dim: i32,
    ) -> Result<(), String> {
        let path = self.path(dim, ExportKind::Area)?;
        let epoch = self.epoch.load(Ordering::SeqCst);
        self.submit(ExportJob::Area { path, layout, regions, cache, dim, epoch })
    }

    // a finished, failed or stopped export, as the message to show
    pub fn take_result(&self) -> Option<String> {
        self.results.try_recv().ok()
    }

    // stops the running export at its next row, then joins the thread
    pub fn shutdown(&self) {
        self.epoch.fetch_add(1, Ordering::SeqCst);
        Shared::lock(&self.jobs).take();
        if let Some(handle) = Shared::lock(&self.worker).take()
            && handle.join().is_err()
        {
            curia::error!("the export thread panicked");
        }
    }

    fn submit(&self, job: ExportJob) -> Result<(), String> {
        if self.busy.swap(true, Ordering::SeqCst) {
            return Err("An export is already running".to_string());
        }
        let sent = Shared::lock(&self.jobs).as_ref().is_some_and(|jobs| jobs.try_send(job).is_ok());
        if !sent {
            self.busy.store(false, Ordering::SeqCst);
            return Err("The export thread has stopped".to_string());
        }
        Ok(())
    }

    fn path(&self, dim: i32, kind: ExportKind) -> Result<PathBuf, String> {
        let world = Shared::lock(&self.world).clone().ok_or_else(|| "No world is open".to_string())?;
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
        Ok(self.dir.join(ExportName::file(&world, dim, secs, kind)))
    }
}

impl Drop for ExportService {
    fn drop(&mut self) {
        self.shutdown();
    }
}
