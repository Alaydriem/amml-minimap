use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender};
use minimap_core::services::export::{AreaLayout, AreaRow};
use minimap_core::structs::pos::RegionPos;
use minimap_core::structs::tile::Tile;
use minimap_store::services::RegionFiles;

use crate::errors::ExportError;
use crate::services::ExportService;
use crate::structs::{ExportJob, Shared};

// runs on the export thread: one job at a time; an area is streamed one row of regions at a time,
// so its memory is one row buffer and one tile
pub(crate) struct ExportWorker {
    shared: Arc<Shared>,
    jobs: Receiver<ExportJob>,
    results: Sender<String>,
    busy: Arc<AtomicBool>,
    epoch: Arc<AtomicU64>,
    tile: Vec<u8>,
    row: Vec<u8>,
}

impl ExportWorker {
    pub(crate) fn new(
        shared: Arc<Shared>,
        jobs: Receiver<ExportJob>,
        results: Sender<String>,
        busy: Arc<AtomicBool>,
        epoch: Arc<AtomicU64>,
    ) -> Self {
        Self { shared, jobs, results, busy, epoch, tile: vec![0; Tile::BYTES], row: Vec::new() }
    }

    pub(crate) fn run(mut self) {
        while let Ok(job) = self.jobs.recv() {
            let path = match &job {
                ExportJob::Area { path, .. } => path.clone(),
            };
            let name = path.file_name().map_or_else(String::new, |n| n.to_string_lossy().into_owned());
            let message = match self.write(job) {
                Ok(()) => format!("{}{name}", ExportService::EXPORTED),
                Err(error) => {
                    if path.exists()
                        && let Err(remove) = fs::remove_file(&path)
                    {
                        curia::warn!("partial export not removed", { path: path.display().to_string(), error: remove.to_string() });
                    }
                    match error {
                        ExportError::Stopped => "Export stopped: the world was left".to_string(),
                        other => {
                            curia::warn!("export failed", { file: name.as_str(), error: other.to_string() });
                            format!("Export failed: {other}")
                        }
                    }
                }
            };
            self.busy.store(false, Ordering::SeqCst);
            if self.results.try_send(message).is_err() {
                curia::warn!("export result dropped: nobody is reading them");
            }
            self.row = Vec::new();
        }
    }

    fn write(&mut self, job: ExportJob) -> Result<(), ExportError> {
        match job {
            ExportJob::Area { path, layout, regions, cache, dim, epoch } => {
                self.write_area(&path, &layout, regions, cache.map(RegionFiles::new), (dim, epoch))
            }
        }
    }

    // at is (dim, epoch); a changed epoch means the world was left
    fn write_area(
        &mut self,
        path: &Path,
        layout: &AreaLayout,
        regions: Vec<RegionPos>,
        files: Option<RegionFiles>,
        at: (i32, u64),
    ) -> Result<(), ExportError> {
        let (dim, epoch) = at;
        let wanted: HashSet<RegionPos> = regions.into_iter().collect();
        let scale = layout.blocks_per_pixel as usize;
        self.row.resize(layout.row_bytes(), 0);
        let mut file = Self::create(path)?;
        let mut writer = Self::encoder(&mut file, layout.width, layout.height).write_header()?;
        {
            let mut stream = writer.stream_writer()?;
            for z in layout.z0..=layout.z1 {
                if self.epoch.load(Ordering::SeqCst) != epoch {
                    return Err(ExportError::Stopped);
                }
                self.row.fill(0);
                for (column, x) in (layout.x0..=layout.x1).enumerate() {
                    let region = RegionPos::new(x, z, dim);
                    if wanted.contains(&region) && self.read_tile(region, files.as_ref())? {
                        AreaRow::place(&mut self.row, layout.width as usize, column, &self.tile, scale);
                    }
                }
                stream.write_all(&self.row)?;
            }
            stream.finish()?;
        }
        writer.finish()?;
        file.flush()?;
        Ok(())
    }

    // the tile in memory first, then the region file; false when neither has pixels
    fn read_tile(&mut self, region: RegionPos, files: Option<&RegionFiles>) -> Result<bool, ExportError> {
        if let Some(tile) = Shared::lock(&self.shared.tiles).peek(region) {
            self.tile.copy_from_slice(tile.pixels());
            return Ok(true);
        }
        let Some(files) = files else { return Ok(false) };
        match files.read(region)? {
            Some(snapshot) if snapshot.has_pixels() => {
                self.tile.copy_from_slice(&snapshot.tile);
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn create(path: &Path) -> Result<BufWriter<File>, ExportError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        Ok(BufWriter::new(File::create(path)?))
    }

    fn encoder(file: &mut BufWriter<File>, width: u32, height: u32) -> png::Encoder<'static, &mut BufWriter<File>> {
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        encoder
    }
}
