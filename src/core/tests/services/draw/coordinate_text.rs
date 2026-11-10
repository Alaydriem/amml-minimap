use minimap_core::services::draw::CoordinateText;

#[test]
fn overworld_positions_divide_by_eight_rounding_toward_negative_infinity() {
    assert_eq!(CoordinateText::linked(0, 15, 16).as_deref(), Some("Nether 1, 2"));
    assert_eq!(CoordinateText::linked(0, -9, -8).as_deref(), Some("Nether -2, -1"));
    assert_eq!(CoordinateText::linked(0, -1, 7).as_deref(), Some("Nether -1, 0"));
}

#[test]
fn nether_positions_multiply_by_eight_and_the_end_has_no_link() {
    assert_eq!(CoordinateText::linked(1, -3, 12).as_deref(), Some("Overworld -24, 96"));
    assert_eq!(CoordinateText::linked(2, 100, 100), None);
}
