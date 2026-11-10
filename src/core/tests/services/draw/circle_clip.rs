use minimap_core::services::draw::CircleClip;

#[test]
fn a_line_through_the_circle_is_cut_at_its_edge() {
    let (a, b) = CircleClip::clip((-20.0, 0.0), (20.0, 0.0), (0.0, 0.0), 10.0).unwrap();
    assert!((a.0 + 10.0).abs() < 1e-4 && (b.0 - 10.0).abs() < 1e-4);
}

#[test]
fn a_line_inside_is_kept_and_a_line_outside_is_dropped() {
    assert_eq!(CircleClip::clip((-1.0, 0.0), (1.0, 0.0), (0.0, 0.0), 10.0), Some(((-1.0, 0.0), (1.0, 0.0))));
    assert_eq!(CircleClip::clip((-20.0, 15.0), (20.0, 15.0), (0.0, 0.0), 10.0), None);
}
