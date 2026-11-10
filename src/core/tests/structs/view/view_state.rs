use minimap_core::structs::view::ViewState;

#[test]
fn the_anchor_wins_over_the_camera_relative_offset() {
    let mut v = ViewState::new(5);
    v.offset_x = 10.0;
    assert_eq!(v.centre((100.0, -50.0)), (110.0, -50.0));
    v.anchor = Some((-1000.5, 2000.5));
    assert_eq!(v.centre((100.0, -50.0)), (-1000.5, 2000.5));
}
