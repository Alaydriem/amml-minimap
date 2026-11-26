use std::thread::JoinHandle;

use crossbeam_channel::Sender;

// a world flush on its own thread; with telemetry on, the leave's game-thread milliseconds go to it for its perf line
pub struct RunningFlush {
    pub thread: JoinHandle<()>,
    pub leave_ms: Option<Sender<u64>>,
}
