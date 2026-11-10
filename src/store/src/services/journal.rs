use std::collections::HashMap;
use std::path::Path;

use minimap_core::structs::pos::{ChunkPos, RegionPos};
use wal_db::{Lsn, SegmentedStore, Wal};

use crate::errors::StoreError;
use crate::services::{RecordCodec, RegionFiles};
use crate::structs::{JournalRecord, RegionSnapshot};

// the append-only log of chunk records for one dimension, with an index of the latest record per chunk
pub struct Journal {
    wal: Wal<SegmentedStore>,
    index: HashMap<ChunkPos, Lsn>,
    head: u64,
    max_bytes: u64,
    scratch: Vec<u8>,
    // bytes this journal has written to disk: records, then region files at compaction
    written: u64,
}

impl Journal {
    pub const SEGMENT_BYTES: u64 = 4 * 1024 * 1024;

    // replays every record after the stored head; records that do not decode are skipped
    pub fn open(dir: &Path, max_bytes: u64) -> Result<Self, StoreError> {
        let wal = Wal::open_segmented(dir, Self::SEGMENT_BYTES)?;
        let mut index = HashMap::new();
        let mut head = None;
        let mut skipped = 0usize;
        for entry in wal.iter()? {
            let record = entry?;
            head.get_or_insert(record.lsn().get());
            match RecordCodec::decode(record.data()) {
                Ok(decoded) => {
                    index.insert(decoded.pos, record.lsn());
                }
                Err(_) => skipped += 1,
            }
        }
        if skipped > 0 {
            curia::warn!("journal records were skipped: unreadable or an older format", { skipped: skipped, dir: dir.display().to_string() });
        }
        let head = head.unwrap_or(wal.len());
        Ok(Self { wal, index, head, max_bytes, scratch: Vec::new(), written: 0 })
    }

    pub fn indexed_len(&self) -> usize {
        self.index.len()
    }

    pub fn live_bytes(&self) -> u64 {
        self.wal.len().saturating_sub(self.head)
    }

    pub fn bytes_written(&self) -> u64 {
        self.written
    }

    pub fn needs_compaction(&self) -> bool {
        self.live_bytes() > self.max_bytes
    }

    pub fn append(&mut self, record: &JournalRecord) -> Result<(), StoreError> {
        let bytes = RecordCodec::encode(record, &mut self.scratch)?;
        let lsn = self.wal.append(&bytes)?;
        self.written += bytes.len() as u64;
        self.index.insert(record.pos, lsn);
        Ok(())
    }

    pub fn sync(&self) -> Result<(), StoreError> {
        Ok(self.wal.sync()?)
    }

    // the latest record of every chunk of the region that has one
    pub fn records_for_region(&self, region: RegionPos, out: &mut Vec<JournalRecord>) -> Result<(), StoreError> {
        out.clear();
        for lz in 0..16 {
            for lx in 0..16 {
                let Some(&lsn) = self.index.get(&region.chunk(lx, lz)) else { continue };
                if let Some(entry) = self.wal.iter_from(lsn)?.next() {
                    out.push(RecordCodec::decode(entry?.data())?);
                }
            }
        }
        Ok(())
    }

    // merges every indexed record into its region file, then drops the merged records
    pub fn compact(&mut self, files: &RegionFiles) -> Result<usize, StoreError> {
        if self.index.is_empty() {
            return Ok(0);
        }
        self.sync()?;
        let mut regions: Vec<RegionPos> = self.index.keys().map(ChunkPos::region).collect();
        regions.sort_unstable();
        regions.dedup();
        let mut records = Vec::with_capacity(256);
        for &region in &regions {
            self.records_for_region(region, &mut records)?;
            let mut snapshot = match files.read(region)? {
                Some(snapshot) => snapshot,
                None => {
                    let mut snapshot = RegionSnapshot::new(region, records.first().map_or(0, |r| r.settings_hash));
                    snapshot.table_id = files.table_id();
                    snapshot
                }
            };
            for record in &records {
                snapshot.apply(record);
            }
            self.written += files.write(&snapshot)? as u64;
        }
        let head = self.wal.truncate_before(Lsn::new(self.wal.len()))?;
        self.head = head.get();
        self.index.clear();
        Ok(regions.len())
    }
}
