use minimap_core::services::draw::TextScale;

#[test]
fn text_is_scaled_for_screens_taller_than_1080_and_never_shrunk() {
    assert_eq!(TextScale::for_display(1080.0), 1.0);
    assert_eq!(TextScale::for_display(2160.0), 2.0);
    assert!((TextScale::for_display(1583.0) - 1.4657).abs() < 1e-3);
    assert_eq!(TextScale::for_display(720.0), 1.0);
}
