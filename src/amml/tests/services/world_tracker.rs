use std::path::PathBuf;

use alaydriem_minimap::types::QueuedColumn;
use amml_api::testing::FakeClient;
use amml_api::{ChunkLoad, ColumnSample, Connection, ConnectionKind, WorldInfo};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::{CursorAction, ScanPlan, SubmitStatus};
use minimap_store::structs::WorldIdentity;

use crate::support::{FakeTerrain, MinimapRig, RecordingSink, SinkCall, TrackerRig};

fn at(x: i32, z: i32) -> ChunkPos {
    ChunkPos { x, z, dim: 0 }
}

fn plan(candidates: Vec<ChunkPos>, limit: usize) -> ScanPlan {
    ScanPlan { candidates, limit, camera_height: 320, transparent_water: true, ..ScanPlan::default() }
}

fn sink(plan: ScanPlan) -> RecordingSink {
    RecordingSink { plan, ..RecordingSink::default() }
}

fn info(folder: &str) -> WorldInfo {
    WorldInfo {
        storage_folder: Some(PathBuf::from(folder)),
        level_name: None,
        seed: Some(5),
        realm: Some(false),
        commands_enabled: None,
        connection: Some(Connection { kind: ConnectionKind::Local, host: String::new(), port: 0 }),
    }
}

fn player_with_info(world_info: Option<WorldInfo>) -> FakeClient {
    FakeClient { world_info, ..MinimapRig::player(0, FakeTerrain::world(&[(0, 0), (1, 0), (2, 0)])) }
}

#[test]
fn the_first_tick_in_a_world_enters_it_named_by_its_folder() {
    let rig = TrackerRig::new(player_with_info(Some(info("C:/worlds/a"))), sink(plan(vec![], 0)));

    rig.tick();

    let world_dir = PathBuf::from("C:/worlds/a").join("@alaydriem").join("minimap").join("data");
    rig.with_sink(|sink| assert_eq!(sink.calls[0], SinkCall::Enter(WorldIdentity::Local { world_dir }, 0)));
    assert!(rig.logged("info minimap world entered dimension=0"));
    assert!(rig.logged("info minimap world named kind=local folder=1"));
}

#[test]
fn a_server_world_is_named_remote() {
    let mut world_info = info("C:/worlds/a");
    world_info.connection = Some(Connection { kind: ConnectionKind::Remote, host: "play.example".into(), port: 19132 });
    let rig = TrackerRig::new(player_with_info(Some(world_info)), sink(plan(vec![], 0)));

    rig.tick();

    assert!(rig.logged("info minimap world named kind=remote folder=0"));
}

#[test]
fn the_tick_gives_the_runtime_the_eye_position_yaw_and_dimension() {
    let rig = TrackerRig::new(player_with_info(None), sink(plan(vec![], 0)));

    rig.tick();

    rig.with_sink(|sink| {
        let player = sink.ticks[0].player;
        assert!(player.present);
        assert_eq!((player.x, player.y, player.z, player.yaw, player.dim), (8.5, 80.0, 8.5, 90.0, 0));
        assert!(sink.ticks[0].mouse_grabbed && sink.ticks[0].cursor_hidden && sink.ticks[0].foreground);
        assert!(!sink.ticks[0].dead);
    });
}

#[test]
fn a_dead_player_is_sent_as_dead() {
    let client = FakeClient { player_alive: Some(false), ..player_with_info(None) };
    let rig = TrackerRig::new(client, sink(plan(vec![], 0)));

    rig.tick();

    rig.with_sink(|sink| assert!(sink.ticks[0].dead));
}

#[test]
fn leaving_the_world_leaves_once_and_ticks_without_a_player() {
    let rig = TrackerRig::new(player_with_info(None), sink(plan(vec![], 0)));
    rig.tick();

    rig.game.set_client(FakeClient { mouse_captured: Some(false), ..FakeClient::default() });
    rig.tick();
    rig.tick();

    rig.with_sink(|sink| {
        assert_eq!(sink.calls.iter().filter(|call| **call == SinkCall::Leave).count(), 1);
        assert!(!sink.ticks[2].player.present);
    });
    assert!(rig.logged("info minimap world left"));
}

#[test]
fn a_world_leave_event_leaves_at_once_and_drops_the_cursor_clip() {
    let clip = ScanPlan { cursor: CursorAction::ClipToCentre, ..plan(vec![], 0) };
    let rig = TrackerRig::new(player_with_info(None), sink(clip));
    rig.tick();

    rig.leave();

    rig.with_sink(|sink| assert_eq!(sink.calls.last(), Some(&SinkCall::Leave)));
    assert_eq!(rig.game.window_requests(), vec!["clip_cursor centre", "clip_cursor none"]);
    assert!(rig.logged("info minimap cursor clip off reason=left"));
}

#[test]
fn a_dimension_change_is_logged_and_sent() {
    let rig = TrackerRig::new(player_with_info(None), sink(plan(vec![], 0)));
    rig.tick();

    rig.game.set_client(FakeClient { dimension: Some(1), ..player_with_info(None) });
    rig.tick();

    rig.with_sink(|sink| assert_eq!(sink.ticks[1].player.dim, 1));
    assert!(rig.logged("info minimap dimension dimension=1"));
}

#[test]
fn probing_stops_reading_after_the_limit_of_loaded_chunks() {
    let rig = TrackerRig::new(player_with_info(None), sink(plan(vec![at(5, 5), at(0, 0), at(1, 0), at(2, 0)], 2)));

    rig.tick();

    rig.with_sink(|sink| {
        assert_eq!(
            sink.submitted_status(),
            vec![
                (at(5, 5), SubmitStatus::NotLoaded),
                (at(0, 0), SubmitStatus::Loaded),
                (at(1, 0), SubmitStatus::Loaded),
                (at(2, 0), SubmitStatus::NotProbed),
            ]
        );
    });
}

#[test]
fn a_chunk_with_a_placeholder_top_is_submitted_to_probe_again() {
    let mut world = FakeTerrain::world(&[]);
    world.chunks.insert((0, 0), FakeTerrain::chunk(ChunkLoad::TopPlaceholder, FakeTerrain::stone()));
    let client = FakeClient { world: Some(world), ..player_with_info(None) };
    let rig = TrackerRig::new(client, sink(plan(vec![at(0, 0)], 4)));

    rig.tick();

    rig.with_sink(|sink| assert_eq!(sink.submitted_status(), vec![(at(0, 0), SubmitStatus::TopPlaceholder)]));
}

#[test]
fn chunks_read_and_the_colour_source_are_logged_once() {
    let candidates: Vec<ChunkPos> = (0..3).map(|x| at(x, 0)).collect();
    let rig = TrackerRig::new(player_with_info(None), sink(plan(candidates, 32)));

    for _ in 0..12 {
        rig.tick();
    }

    assert_eq!(rig.logged_start("info minimap chunks read loaded="), 1);
    assert!(rig.logged("info minimap chunks read loaded=33"));
    assert_eq!(rig.logged_start("info minimap colours source=vanilla"), 1);
}

#[test]
fn a_requested_biome_is_named_and_no_biome_gets_an_empty_name() {
    let requests = vec![FakeTerrain::PLAINS, ColumnSample::NO_BIOME];
    let rig =
        TrackerRig::new(player_with_info(None), sink(ScanPlan { biome_requests: requests, ..plan(vec![at(0, 0)], 4) }));

    rig.tick();

    rig.with_sink(|sink| {
        assert!(sink.calls.contains(&SinkCall::BiomeName(FakeTerrain::PLAINS, "plains".to_string())));
        assert!(sink.calls.contains(&SinkCall::BiomeName(ColumnSample::NO_BIOME, String::new())));
    });
    assert!(rig.logged("info minimap biome named id=1 name=plains"));
}

#[test]
fn a_block_change_reads_its_column_at_the_next_tick() {
    let mut sink = sink(plan(vec![], 0));
    sink.column_requests.insert((3, 2), (at(0, 0), 35));
    let rig = TrackerRig::new(player_with_info(None), sink);
    rig.tick();

    rig.block_changed(QueuedColumn { dimension: 0, x: 3, z: 2 });
    rig.with_sink(|sink| assert!(!sink.calls.iter().any(|call| matches!(call, SinkCall::Column(..)))));
    rig.tick();

    rig.with_sink(|sink| {
        assert!(sink.calls.contains(&SinkCall::BlockChanged(3, 2, 0)));
        assert!(sink.calls.iter().any(|call| matches!(call, SinkCall::Column(pos, 35, _) if *pos == at(0, 0))));
    });
    assert!(rig.logged("info minimap block change column x=3 z=2"));
}

#[test]
fn a_block_change_of_another_dimension_is_not_read() {
    let rig = TrackerRig::new(player_with_info(None), sink(plan(vec![], 0)));
    rig.tick();

    rig.block_changed(QueuedColumn { dimension: 1, x: 3, z: 2 });
    rig.tick();

    rig.with_sink(|sink| assert!(!sink.calls.contains(&SinkCall::BlockChanged(3, 2, 1))));
}

#[test]
fn a_clip_the_plan_asks_for_in_play_is_requested() {
    let clip = ScanPlan { cursor: CursorAction::ClipToCentre, ..plan(vec![], 0) };
    let rig = TrackerRig::new(player_with_info(None), sink(clip));

    rig.tick();

    assert_eq!(rig.game.window_requests(), vec!["clip_cursor centre"]);
    assert!(rig.logged("info minimap cursor clip on"));
}
