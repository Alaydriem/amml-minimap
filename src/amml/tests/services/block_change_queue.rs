use alaydriem_minimap::services::BlockChangeQueue;
use alaydriem_minimap::types::QueuedColumn;

fn column(x: i32) -> QueuedColumn {
    QueuedColumn { dimension: 0, x, z: 5 }
}

#[test]
fn a_column_queued_twice_is_read_once() {
    let mut queue = BlockChangeQueue::new();

    queue.push(column(1));
    queue.push(column(1));

    assert_eq!(queue.pop(), Some(column(1)));
    assert_eq!(queue.pop(), None);
}

#[test]
fn a_change_past_the_capacity_is_dropped_and_counted() {
    let mut queue = BlockChangeQueue::new();
    for x in 0..BlockChangeQueue::CAPACITY as i32 {
        assert!(queue.push(column(x)));
    }

    let kept = queue.push(column(-1));

    assert!(!kept);
    assert_eq!(queue.len(), BlockChangeQueue::CAPACITY);
    assert_eq!(queue.dropped(), 1);
    assert_eq!(queue.pop(), Some(column(0)));
}
