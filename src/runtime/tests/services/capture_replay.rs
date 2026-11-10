use minimap_runtime::services::{CaptureReader, CaptureReplay, Runtime};

use crate::support::SyntheticWorld;

#[test]
fn a_recorded_session_replays_into_the_same_chunk_store() {
    let mut world = SyntheticWorld::new();
    world.config.debug.capture_probes = true;
    let runtime = world.runtime();
    for _ in 0..40 {
        world.tick(&runtime);
        world.draw(&runtime);
    }
    let recorded = runtime.stats();
    runtime.shutdown();
    let captures = world.paths().data_dir.join(Runtime::CAPTURE_DIR);
    let file = std::fs::read_dir(&captures).unwrap().next().unwrap().unwrap().path();

    let replay_world = SyntheticWorld::new();
    let replayed = Runtime::new(replay_world.config.clone(), replay_world.paths(), replay_world.grant).unwrap();
    let mut reader = CaptureReader::open(&file).unwrap();
    let mut replay = CaptureReplay::new(replay_world.identity());
    let applied = replay.run(&replayed, &mut reader).unwrap();
    // enter + 40 ticks + 40 frames + every submit; the leave at shutdown is recorded too
    assert!(applied > 81, "{applied}");
    let stats = replayed.stats();
    replayed.shutdown();
    assert_eq!(stats.chunks_submitted, recorded.chunks_submitted);
    assert_eq!(stats.surfaces_resident, recorded.surfaces_resident);
}
