use std::collections::{BTreeSet, HashMap};

use crate::services::scan::ScanOutcome;
use crate::services::scan::scan_key::ScanKey;
use crate::structs::pos::ChunkPos;
use crate::traits::ScanHistory;

pub struct ScanScheduler {
    center: Option<ChunkPos>,
    radius: i32,
    rescan_interval: u64,
    queue: BTreeSet<ScanKey>,
    // every chunk in the square; a key absent from `queue` is in flight
    entries: HashMap<(i32, i32), ScanKey>,
}

impl ScanScheduler {
    pub const RETRY_TICKS: u64 = 20;
    pub const MAX_RADIUS: i32 = 64;

    pub fn new(rescan_interval: u64) -> Self {
        Self { center: None, radius: 0, rescan_interval, queue: BTreeSet::new(), entries: HashMap::new() }
    }

    pub fn clear(&mut self) {
        self.center = None;
        self.queue.clear();
        self.entries.clear();
    }

    pub fn tracked_len(&self) -> usize {
        self.entries.len()
    }

    pub fn update_center(&mut self, center: ChunkPos, radius: i32, tick: u64) {
        let radius = radius.clamp(0, Self::MAX_RADIUS);
        match self.center {
            Some(old) if old == center && self.radius == radius => {}
            Some(old) if old.dim == center.dim && self.radius == radius => self.shift(old, center, tick),
            _ => self.rebuild(center, radius, tick),
        }
    }

    // pops due chunks in queue order; chunks scanned too recently are requeued, not returned
    pub fn take_due(&mut self, tick: u64, cap: usize, history: &impl ScanHistory, out: &mut Vec<ChunkPos>) {
        out.clear();
        let Some(center) = self.center else { return };
        while out.len() < cap {
            let Some(key) = self.queue.first().copied() else { break };
            if key.due > tick {
                break;
            }
            self.queue.remove(&key);
            let pos = ChunkPos::new(key.x, key.z, center.dim);
            if let Some(last) = history.last_scan(pos)
                && tick.saturating_sub(last) < self.rescan_interval
            {
                self.requeue(key, last + self.rescan_interval);
                continue;
            }
            out.push(pos);
        }
    }

    pub fn report(&mut self, pos: ChunkPos, outcome: ScanOutcome, tick: u64) {
        let Some(key) = self.entries.get(&(pos.x, pos.z)).copied() else { return };
        if self.center.map(|c| c.dim) != Some(pos.dim) || self.queue.contains(&key) {
            return;
        }
        let due = match outcome {
            ScanOutcome::Scanned { needs_retry: false } => tick + self.rescan_interval,
            ScanOutcome::Scanned { needs_retry: true } | ScanOutcome::NotLoaded | ScanOutcome::TopPlaceholder => {
                tick + Self::RETRY_TICKS
            }
            ScanOutcome::NotProbed => key.due,
        };
        self.requeue(key, due);
    }

    fn requeue(&mut self, key: ScanKey, due: u64) {
        let next = ScanKey::new(due, key.rank, key.x, key.z);
        self.queue.insert(next);
        self.entries.insert((key.x, key.z), next);
    }

    fn rebuild(&mut self, center: ChunkPos, radius: i32, tick: u64) {
        self.queue.clear();
        self.entries.clear();
        self.center = Some(center);
        self.radius = radius;
        for dz in -radius..=radius {
            for dx in -radius..=radius {
                self.insert(center, dx, dz, tick);
            }
        }
    }

    fn shift(&mut self, old: ChunkPos, center: ChunkPos, tick: u64) {
        let r = self.radius;
        self.entries.retain(|&(x, z), key| {
            let inside = (x - center.x).abs() <= r && (z - center.z).abs() <= r;
            if !inside {
                self.queue.remove(key);
            }
            inside
        });
        self.center = Some(center);
        for dz in -r..=r {
            for dx in -r..=r {
                let (x, z) = (center.x + dx, center.z + dz);
                let was_inside = (x - old.x).abs() <= r && (z - old.z).abs() <= r;
                if !was_inside {
                    self.insert(center, dx, dz, tick);
                }
            }
        }
    }

    fn insert(&mut self, center: ChunkPos, dx: i32, dz: i32, tick: u64) {
        let key = ScanKey::new(tick, ScanKey::spiral_rank(dx, dz, self.radius), center.x + dx, center.z + dz);
        self.queue.insert(key);
        self.entries.insert((key.x, key.z), key);
    }
}
