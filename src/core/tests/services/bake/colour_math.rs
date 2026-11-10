use minimap_core::services::bake::ColourMath;
use minimap_core::structs::Rgba;

#[test]
fn multiply_truncates_clamps_and_keeps_alpha() {
    assert_eq!(ColourMath::multiply(Rgba::new(255, 100, 3, 7), 0.5), Rgba::new(127, 50, 1, 7));
    assert_eq!(ColourMath::multiply(Rgba::new(200, 10, 0, 255), 1.5), Rgba::new(255, 15, 0, 255));
}

#[test]
fn blend_mixes_by_opacity_and_keeps_destination_alpha() {
    let blended = ColourMath::blend(Rgba::new(100, 100, 100, 9), Rgba::new(0, 0, 200, 255), 0.5);
    assert_eq!(blended, Rgba::new(50, 50, 150, 9));
}

#[test]
fn scale_ratio_uses_integer_division_and_clamps() {
    assert_eq!(ColourMath::scale_ratio(Rgba::new(100, 200, 7, 1), 130, 100), Rgba::new(130, 255, 9, 1));
    assert_eq!(ColourMath::scale_ratio(Rgba::new(100, 200, 7, 1), 100, 130), Rgba::new(76, 153, 5, 1));
}
