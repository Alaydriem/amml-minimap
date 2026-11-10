use minimap_core::services::draw::OverlayGate;

#[test]
fn the_gate_opens_at_six_pixels_per_chunk() {
    assert!(OverlayGate::open(0.375));
    assert!(!OverlayGate::open(0.37));
    assert_eq!(OverlayGate::MIN_CHUNK_PIXELS, 6.0);
}
