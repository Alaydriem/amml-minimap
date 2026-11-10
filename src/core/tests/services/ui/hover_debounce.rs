use minimap_core::services::ui::HoverDebounce;
use minimap_core::structs::pos::RegionPos;

#[test]
fn a_region_is_asked_for_after_the_cursor_rests_on_it_for_eight_frames() {
    let mut d = HoverDebounce::new();
    let r = RegionPos::new(3, -2, 0);
    assert!((0..7).all(|_| !d.observe(r)));
    assert!(d.observe(r));
    assert!((0..20).all(|_| !d.observe(r)));
    assert!(!d.observe(RegionPos::new(4, -2, 0)));
}

#[test]
fn a_reset_lets_the_same_region_be_asked_again() {
    let mut d = HoverDebounce::new();
    let r = RegionPos::new(0, 0, 0);
    assert!((0..8).any(|_| d.observe(r)));
    d.reset();
    assert!((0..8).any(|_| d.observe(r)));
}
