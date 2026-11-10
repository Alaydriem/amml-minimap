use minimap_core::services::dirty::RegionDebounce;

fn ticks_until_fire(debounce: &mut RegionDebounce, mark_every: Option<u32>) -> u32 {
    for tick in 1..=100 {
        if let Some(every) = mark_every
            && tick % every == 0
        {
            debounce.mark();
        }
        if debounce.tick() {
            return tick;
        }
    }
    0
}

#[test]
fn fires_ten_ticks_after_a_single_mark() {
    assert_eq!(ticks_until_fire(&mut RegionDebounce::new(), None), 10);
}

#[test]
fn repeated_marks_delay_firing_but_never_past_sixty_ticks() {
    assert_eq!(ticks_until_fire(&mut RegionDebounce::new(), Some(5)), 60);
}

#[test]
fn a_fired_debounce_does_not_fire_again() {
    let mut debounce = RegionDebounce::new();
    ticks_until_fire(&mut debounce, None);
    assert!(!debounce.tick());
}
