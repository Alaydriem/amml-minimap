use std::sync::{Arc, Mutex};

use alaydriem_minimap::services::WorldTracker;
use alaydriem_minimap::types::QueuedColumn;
use amml_api::amml_abi::ClientTick;
use amml_api::testing::{FakeClient, FakeGame};
use amml_api::{HookOrder, Listener};
use tempfile::TempDir;

use crate::support::{MinimapRig, RecordingSink};

// a world tracker over a recording sink, ticked by a listener of the fake game's client tick
pub struct TrackerRig {
    pub game: FakeGame,
    pub tracker: Arc<Mutex<WorldTracker<RecordingSink>>>,
    _dir: TempDir,
    _listener: Listener,
}

impl TrackerRig {
    pub fn new(client: FakeClient, sink: RecordingSink) -> Self {
        let game = MinimapRig::game();
        game.set_client(client);
        let dir = tempfile::tempdir().unwrap();
        let ctx = MinimapRig::ctx(&game);
        let tracker = Arc::new(Mutex::new(WorldTracker::new(&ctx, sink, dir.path())));
        let ticking = Arc::clone(&tracker);
        let listener = ctx
            .events()
            .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
                if let Some(client) = at.client() {
                    ticking.lock().unwrap().tick(&client);
                }
            })
            .unwrap();
        Self { game, tracker, _dir: dir, _listener: listener }
    }

    pub fn tick(&self) {
        MinimapRig::tick(&self.game);
    }

    pub fn with_sink<R>(&self, f: impl FnOnce(&mut RecordingSink) -> R) -> R {
        f(self.tracker.lock().unwrap().sink_mut())
    }

    pub fn block_changed(&self, column: QueuedColumn) {
        self.tracker.lock().unwrap().block_changed(column);
    }

    pub fn leave(&self) {
        self.tracker.lock().unwrap().leave();
    }

    pub fn logged(&self, line: &str) -> bool {
        self.game.logs().iter().any(|logged| logged == line)
    }

    pub fn logged_start(&self, start: &str) -> usize {
        self.game.logs().iter().filter(|logged| logged.starts_with(start)).count()
    }
}
