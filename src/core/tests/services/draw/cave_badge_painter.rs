use minimap_core::services::draw::CaveBadgePainter;
use minimap_core::structs::draw::{DrawCommand, DrawList, TextAlign};

#[test]
fn the_badge_names_the_band_on_a_band_centred_under_its_anchor() {
    let mut draw = DrawList::new();
    CaveBadgePainter::paint(&mut draw, (500.0, 20.0), 16.0, (32, 47));
    let DrawCommand::RectFilled { p0, p1, .. } = draw.commands[0] else { panic!("a band first") };
    assert!(((p0.0 + p1.0) * 0.5 - 500.0).abs() < 1e-3 && p0.1 == 20.0 && p1.1 > 36.0);
    let DrawCommand::Text { start, len, align, pos, .. } = draw.commands[1] else { panic!("then the text") };
    assert_eq!(draw.text_of(start, len), "CAVE \u{b7} Y 32 to 47");
    assert_eq!((align, pos.0), (TextAlign::Centre, 500.0));
}
