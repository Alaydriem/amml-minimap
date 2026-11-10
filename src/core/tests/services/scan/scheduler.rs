use minimap_core::services::scan::{ScanOutcome, ScanScheduler};
use minimap_core::structs::pos::ChunkPos;

use crate::support::FakeEnvironment;

fn take(scheduler: &mut ScanScheduler, tick: u64, cap: usize, history: &FakeEnvironment) -> Vec<ChunkPos> {
    let mut out = Vec::new();
    scheduler.take_due(tick, cap, history, &mut out);
    out
}

#[test]
fn a_fresh_square_is_scanned_centre_first_in_spiral_order() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(10, 20, 0), 2, 0);
    let order = take(&mut scheduler, 0, 5, &FakeEnvironment::new());
    let offsets: Vec<(i32, i32)> = order.iter().map(|p| (p.x - 10, p.z - 20)).collect();
    assert_eq!(offsets, vec![(0, 0), (0, -1), (-1, 0), (1, 0), (0, 1)]);
}

#[test]
fn moving_east_drops_the_west_column_and_adds_the_east_column() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(0, 0, 0), 1, 0);
    take(&mut scheduler, 0, 9, &FakeEnvironment::new());
    scheduler.update_center(ChunkPos::new(1, 0, 0), 1, 5);
    assert_eq!(scheduler.tracked_len(), 9);
    let fresh = take(&mut scheduler, 5, 9, &FakeEnvironment::new());
    let mut xs: Vec<i32> = fresh.iter().map(|p| p.x).collect();
    xs.dedup();
    assert_eq!(xs, vec![2]);
    assert_eq!(fresh.len(), 3);
}

#[test]
fn report_outcomes_set_the_next_due_tick() {
    let mut scheduler = ScanScheduler::new(100);
    let pos = ChunkPos::new(0, 0, 0);
    scheduler.update_center(pos, 0, 0);
    let history = FakeEnvironment::new();

    assert_eq!(take(&mut scheduler, 0, 1, &history), vec![pos]);
    scheduler.report(pos, ScanOutcome::NotLoaded, 0);
    assert!(take(&mut scheduler, 19, 1, &history).is_empty());
    assert_eq!(take(&mut scheduler, 20, 1, &history), vec![pos]);

    scheduler.report(pos, ScanOutcome::Scanned { needs_retry: false }, 20);
    assert!(take(&mut scheduler, 119, 1, &history).is_empty());
    assert_eq!(take(&mut scheduler, 120, 1, &history), vec![pos]);

    scheduler.report(pos, ScanOutcome::Scanned { needs_retry: true }, 120);
    assert_eq!(take(&mut scheduler, 140, 1, &history), vec![pos]);
}

#[test]
fn not_probed_candidates_return_to_the_front_of_the_queue() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(0, 0, 0), 1, 0);
    let history = FakeEnvironment::new();
    let first = take(&mut scheduler, 0, 3, &history);
    scheduler.report(first[0], ScanOutcome::Scanned { needs_retry: false }, 0);
    scheduler.report(first[1], ScanOutcome::NotProbed, 0);
    scheduler.report(first[2], ScanOutcome::NotProbed, 0);
    assert_eq!(take(&mut scheduler, 1, 2, &history), vec![first[1], first[2]]);
}

#[test]
fn a_recently_scanned_chunk_is_requeued_without_being_returned() {
    let mut scheduler = ScanScheduler::new(1200);
    let pos = ChunkPos::new(3, 3, 0);
    scheduler.update_center(pos, 0, 1000);
    let mut history = FakeEnvironment::new();
    history.scans.insert(pos, 500);
    assert!(take(&mut scheduler, 1000, 1, &history).is_empty());
    assert!(take(&mut scheduler, 1699, 1, &history).is_empty());
    assert_eq!(take(&mut scheduler, 1700, 1, &history), vec![pos]);
}

#[test]
fn reports_for_chunks_outside_the_square_are_ignored() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(0, 0, 0), 0, 0);
    take(&mut scheduler, 0, 1, &FakeEnvironment::new());
    scheduler.update_center(ChunkPos::new(5, 0, 0), 0, 1);
    scheduler.report(ChunkPos::new(0, 0, 0), ScanOutcome::NotLoaded, 1);
    assert_eq!(scheduler.tracked_len(), 1);
}

#[test]
fn a_dimension_change_rebuilds_the_square() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(0, 0, 0), 1, 0);
    take(&mut scheduler, 0, 9, &FakeEnvironment::new());
    scheduler.update_center(ChunkPos::new(0, 0, 1), 1, 3);
    let fresh = take(&mut scheduler, 3, 9, &FakeEnvironment::new());
    assert_eq!(fresh.len(), 9);
    assert!(fresh.iter().all(|p| p.dim == 1));
}

#[test]
fn a_radius_above_the_maximum_is_clamped_and_still_spirals_from_the_centre() {
    let mut scheduler = ScanScheduler::new(1200);
    scheduler.update_center(ChunkPos::new(0, 0, 0), 500, 0);
    let side = (2 * ScanScheduler::MAX_RADIUS + 1) as usize;
    assert_eq!(scheduler.tracked_len(), side * side);
    let first = take(&mut scheduler, 0, 1, &FakeEnvironment::new());
    assert_eq!(first, vec![ChunkPos::new(0, 0, 0)]);
}
