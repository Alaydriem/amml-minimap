use alaydriem_minimap::services::MapSession;
use alaydriem_minimap::traits::MapSink;
use bytemuck::Zeroable;
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_core::structs::view::PlayerSnapshot;
use minimap_link::errors::SessionError;
use minimap_runtime::structs::{CursorAction, ProbeRequest, ScanPlan, SubmitStatus, TickInput};

use crate::support::TestSession;

fn tick_input() -> TickInput {
    TickInput {
        player: PlayerSnapshot { present: true, x: 8.5, y: 70.0, z: 8.5, yaw: 0.0, dim: 0 },
        foreground: true,
        cursor_hidden: true,
        ..TickInput::default()
    }
}

// probes every candidate as flat ground, the way the column probe hands it over
fn probe_all(session: &mut TestSession, plan: &ScanPlan) {
    for (i, pos) in plan.candidates.iter().enumerate() {
        let mut probe = ChunkProbe::zeroed();
        if i >= plan.limit {
            session.sink().submit_chunk(*pos, SubmitStatus::NotProbed, &probe);
            continue;
        }
        for c in 0..256 {
            probe.surface.surface[c] = 0xFF40_8040;
            probe.surface.height[c] = 64;
            probe.surface.solid[c] = 64;
            probe.biome_ids[c] = 1;
        }
        session.sink().submit_chunk(*pos, SubmitStatus::Loaded, &probe);
    }
}

#[test]
fn a_tick_returns_a_plan_and_probes_reach_the_chunk_store() {
    let mut session = TestSession::new();
    session.enter();
    let mut plan = ScanPlan::default();

    session.sink().tick(&tick_input(), &mut plan);
    probe_all(&mut session, &plan);

    assert!(!plan.candidates.is_empty());
    assert_eq!(plan.limit, 32);
    assert_eq!(plan.camera_height, 320);
    assert_eq!(plan.cursor, CursorAction::ClipToCentre);
    let stats = session.sink().runtime().stats();
    assert_eq!(stats.chunks_submitted, plan.candidates.len().min(plan.limit) as u64);
    assert!(stats.surfaces_resident > 0);
}

#[test]
fn block_changes_ask_for_a_column_of_a_known_chunk() {
    let mut session = TestSession::new();
    session.enter();
    let mut plan = ScanPlan::default();
    session.sink().tick(&tick_input(), &mut plan);
    probe_all(&mut session, &plan);

    let request = session.sink().block_changed(5, 6, 0);

    assert_eq!(request, ProbeRequest::Column { pos: ChunkPos::new(0, 0, 0), index: 6 * 16 + 5 });
    let sample = ColumnSample::new(1, 0, 90, 90, false, 0);
    session.sink().submit_column(ChunkPos::new(0, 0, 0), 6 * 16 + 5, sample);
}

#[test]
fn the_sink_carries_the_block_colour_overrides_for_the_probe() {
    let mut session = TestSession::new();
    session.enter();

    let colours = session.sink().block_colors().to_vec();

    assert_eq!(colours, vec![("minecraft:lava".to_string(), u32::from_le_bytes([230, 110, 20, 255]))]);
}

#[test]
fn the_plan_lists_biome_ids_to_name_and_a_biome_name_answers_them() {
    let mut session = TestSession::new();
    session.enter();
    let mut plan = ScanPlan::default();
    session.sink().tick(&tick_input(), &mut plan);
    probe_all(&mut session, &plan);

    session.sink().tick(&tick_input(), &mut plan);
    let asked = plan.biome_requests.clone();
    session.sink().biome_name(1, "plains");
    probe_all(&mut session, &plan);
    session.sink().tick(&tick_input(), &mut plan);

    assert_eq!(asked, vec![1]);
    assert!(plan.biome_requests.is_empty());
}

#[test]
fn entering_and_leaving_a_world_shows_in_the_stats() {
    let mut session = TestSession::new();
    session.enter();
    let entered = session.sink().runtime().stats();

    session.sink().leave_world();

    assert!(entered.in_world);
    assert!(entered.surface_capacity > 0);
    assert!(!session.sink().runtime().stats().in_world);
}

#[test]
fn connecting_without_a_manager_is_manager_missing() {
    let dir = tempfile::tempdir().unwrap();

    let result = MapSession::connect(&dir.path().join("config"), &dir.path().join("data"));

    assert!(matches!(result, Err(SessionError::Link(amm_core::errors::LinkError::ManagerMissing))));
}
