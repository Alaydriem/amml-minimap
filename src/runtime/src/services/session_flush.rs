use std::time::Duration;

use minimap_store::services::StoreService;
use minimap_store::structs::{JournalRecord, ShutdownReport, StoreCommand};

// the store work of a world that was left: the biome key table, then every record not yet journaled, go to the
// world's store, which then syncs, compacts and stops. It blocks, so it runs off the game thread; dropped without a
// run, it runs then
pub struct SessionFlush {
    store: StoreService,
    keys: Option<(u64, Vec<String>)>,
    records: Vec<Box<JournalRecord>>,
    ran: bool,
}

impl SessionFlush {
    pub const SHUTDOWN: Duration = Duration::from_secs(5);

    pub fn new(store: StoreService, keys: Option<(u64, Vec<String>)>, records: Vec<Box<JournalRecord>>) -> Self {
        Self { store, keys, records, ran: false }
    }

    pub fn records(&self) -> usize {
        self.records.len()
    }

    // the table goes first and is never dropped: the records may use its newest keys
    pub fn run(&mut self) -> ShutdownReport {
        if self.ran {
            return ShutdownReport { finished: true };
        }
        self.ran = true;
        if let Some((table_id, names)) = self.keys.take()
            && self.store.send(StoreCommand::WriteBiomeKeys { table_id, names }).is_err()
        {
            curia::warn!("the store stopped before the biome key table was saved");
        }
        for record in std::mem::take(&mut self.records) {
            if self.store.send(StoreCommand::Append { record, token: StoreCommand::NO_ACK }).is_err() {
                curia::warn!("the store stopped before every chunk was journaled");
                break;
            }
        }
        let report = self.store.shutdown(Self::SHUTDOWN);
        if !report.finished {
            curia::warn!("the store did not finish in time", { timeout_ms: Self::SHUTDOWN.as_millis() as u64 });
        }
        report
    }
}

impl Drop for SessionFlush {
    fn drop(&mut self) {
        self.run();
    }
}
