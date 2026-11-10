use minimap_core::structs::chunk::ShadowMask;

#[test]
fn size_is_one_bit_per_sub_pixel() {
    assert_eq!(ShadowMask::byte_len(2), 128);
    assert_eq!(ShadowMask::byte_len(16), 8192);
}

#[test]
fn set_bits_read_back_as_darkness_and_reset_clears_them() {
    let mut mask = ShadowMask::new(2);
    mask.set(31, 31);
    mask.set(0, 1);
    assert_eq!(mask.value(31, 31), ShadowMask::DARKNESS);
    assert_eq!(mask.value(0, 1), ShadowMask::DARKNESS);
    assert_eq!(mask.value(1, 0), 0.0);
    mask.reset(2);
    assert!(!mask.get(31, 31));
}

#[test]
fn unbaked_mask_is_not_baked_at_any_scale() {
    assert!(!ShadowMask::new(0).is_baked_at(0));
    assert!(ShadowMask::new(4).is_baked_at(4));
    assert!(!ShadowMask::new(4).is_baked_at(2));
}
