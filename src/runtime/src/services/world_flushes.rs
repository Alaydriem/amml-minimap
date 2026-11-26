use std::time::{Duration, Instant};

use crossbeam_channel::Receiver;

use crate::services::SessionFlush;
use crate::structs::RunningFlush;

// the flushes of worlds that were left, each on its own thread, so a leave does not wait for the disk. A later store
// of the same folder waits for its flush through the folder's turn, not here. At most CAPACITY are kept; a hand-off
// past that waits up to HAND_OFF_WAIT for the oldest, then moves it to `left_alone`, which shutdown still waits for.
// `left_alone` holds at most LEFT_ALONE_CAPACITY; past that its oldest handle is dropped and that flush is no longer
// waited for
pub struct WorldFlushes {
    running: Vec<RunningFlush>,
    left_alone: Vec<RunningFlush>,
}

impl WorldFlushes {
    pub const CAPACITY: usize = 4;
    // a cap on a whole flush at shutdown: the record sends, which have no cap of their own, then the store's 5 s stop
    pub const WAIT: Duration = Duration::from_secs(10);
    // the longest a hand-off past CAPACITY waits on the game thread for the oldest flush
    pub const HAND_OFF_WAIT: Duration = Duration::from_secs(1);
    pub const LEFT_ALONE_CAPACITY: usize = 16;
    const POLL: Duration = Duration::from_millis(5);
    const LEAVE_REPORT: Duration = Duration::from_secs(1);

    pub fn new() -> Self {
        Self { running: Vec::with_capacity(Self::CAPACITY), left_alone: Vec::new() }
    }

    // a flush whose thread cannot start runs here, when the closure that owns it is dropped
    pub fn hand_off(&mut self, flush: SessionFlush, telemetry: bool) {
        self.reap();
        if self.running.len() >= Self::CAPACITY {
            let oldest = self.running.remove(0);
            let deadline = Instant::now() + Self::HAND_OFF_WAIT;
            while !oldest.thread.is_finished() && Instant::now() < deadline {
                std::thread::sleep(Self::POLL);
            }
            if oldest.thread.is_finished() {
                let _ = oldest.thread.join();
            } else {
                curia::warn!("too many world flushes are running; the leave no longer waits for the oldest", {
                    capacity: Self::CAPACITY as u64,
                    wait_ms: Self::HAND_OFF_WAIT.as_millis() as u64,
                });
                self.leave_alone(oldest);
            }
        }
        let (leave_ms, report) = if telemetry {
            let (tx, rx) = crossbeam_channel::bounded(1);
            (Some(tx), Some(rx))
        } else {
            (None, None)
        };
        let spawned =
            std::thread::Builder::new().name("minimap-flush".to_string()).spawn(move || Self::run(flush, report));
        match spawned {
            Ok(thread) => self.running.push(RunningFlush { thread, leave_ms }),
            Err(error) => curia::warn!("the world flush ran on the game thread", { error: error.to_string() }),
        }
    }

    // the game-thread time of the leave that handed off the newest flush, for its perf line
    pub fn report_leave(&self, game: Duration) {
        if let Some(sender) = self.running.last().and_then(|flush| flush.leave_ms.as_ref()) {
            let _ = sender.try_send(game.as_millis() as u64);
        }
    }

    // every flush still running, also those a hand-off no longer waited for
    pub fn running(&mut self) -> usize {
        self.reap();
        self.running.len() + self.left_alone.len()
    }

    // waits up to `timeout` for every flush; one still running after it is left to finish alone
    pub fn wait_all(&mut self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while self.running() > 0 && Instant::now() < deadline {
            std::thread::sleep(Self::POLL);
        }
        let left = self.running.len() + self.left_alone.len();
        if left > 0 {
            curia::warn!("world flushes did not finish in time; they are left to finish alone", {
                running: left as u64,
                timeout_ms: timeout.as_millis() as u64,
            });
        }
        self.running.clear();
        self.left_alone.clear();
        left == 0
    }

    fn leave_alone(&mut self, flush: RunningFlush) {
        Self::reap_list(&mut self.left_alone);
        if self.left_alone.len() >= Self::LEFT_ALONE_CAPACITY {
            self.left_alone.remove(0);
            curia::warn!("a world flush is no longer waited for, even at shutdown", {
                capacity: Self::LEFT_ALONE_CAPACITY as u64,
            });
        }
        self.left_alone.push(flush);
    }

    fn reap(&mut self) {
        Self::reap_list(&mut self.running);
        Self::reap_list(&mut self.left_alone);
    }

    fn reap_list(list: &mut Vec<RunningFlush>) {
        let mut i = 0;
        while i < list.len() {
            if list[i].thread.is_finished() {
                let _ = list.remove(i).thread.join();
            } else {
                i += 1;
            }
        }
    }

    fn run(mut flush: SessionFlush, report: Option<Receiver<u64>>) {
        let started = Instant::now();
        let records = flush.records() as u64;
        let finished = flush.run().finished;
        let finish_ms = started.elapsed().as_millis() as u64;
        let Some(report) = report else { return };
        match report.recv_timeout(Self::LEAVE_REPORT) {
            Ok(leave_game_ms) => curia::info!("perf", {
                event: "world_leave",
                leave_game_ms: leave_game_ms,
                finish_ms: finish_ms,
                records: records,
                finished: finished,
            }),
            Err(_) => curia::info!("perf", {
                event: "world_leave",
                finish_ms: finish_ms,
                records: records,
                finished: finished,
            }),
        }
    }
}

impl Default for WorldFlushes {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for WorldFlushes {
    fn drop(&mut self) {
        if !self.running.is_empty() || !self.left_alone.is_empty() {
            self.wait_all(Self::WAIT);
        }
    }
}
