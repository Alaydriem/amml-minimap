use minimap_core::structs::chunk::DirtyFlags;

#[test]
fn clear_full_keeps_a_mark_that_arrived_after_the_snapshot() {
    let mut flags = DirtyFlags::new();
    flags.mark_full();
    let snapshot = flags.epochs();
    flags.mark_full();
    assert!(!flags.clear_full_if(snapshot.full));
    assert!(flags.is_full());
    assert!(flags.clear_full_if(flags.epochs().full));
    assert!(!flags.is_full());
}

#[test]
fn soft_and_full_epochs_are_independent() {
    let mut flags = DirtyFlags::new();
    flags.mark_soft();
    let snapshot = flags.epochs();
    flags.mark_full();
    assert!(flags.clear_soft_if(snapshot.soft));
    assert!(!flags.is_soft());
    assert!(flags.is_full());
}
