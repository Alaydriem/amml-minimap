use minimap_core::structs::chunk::ChunkSurface;
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::services::JournalOutbox;
use minimap_runtime::structs::JournalAck;
use minimap_store::structs::JournalRecord;

fn record(x: i32) -> (JournalRecord, JournalAck) {
    let pos = ChunkPos::new(x, 0, 0);
    (JournalRecord::new(pos, 1, ChunkSurface::empty(), None), JournalAck { pos, journal_version: 1, tile: None })
}

#[test]
fn a_full_outbox_drops_its_oldest_record() {
    let mut outbox = JournalOutbox::new(2);
    for x in 0..3 {
        let (record, ack) = record(x);
        outbox.push(record, ack);
    }
    assert_eq!(outbox.len(), 2);
    assert_eq!(outbox.dropped(), 1);
}

#[test]
fn an_unknown_token_has_no_ack() {
    let mut outbox = JournalOutbox::new(2);
    assert_eq!(outbox.acknowledge(99), None);
    assert_eq!(outbox.pending_acks(), 0);
}

#[test]
fn taking_everything_empties_the_outbox_in_order() {
    let mut outbox = JournalOutbox::new(8);
    for x in 0..3 {
        let (record, ack) = record(x);
        outbox.push(record, ack);
    }
    let taken: Vec<i32> = outbox.take_all().iter().map(|r| r.pos.x).collect();
    assert_eq!(taken, vec![0, 1, 2]);
    assert!(outbox.is_empty());
}
