use minimap_core::services::cave::LayerIds;

#[test]
fn surface_layers_keep_their_dimension_and_cave_bands_map_both_ways() {
    assert_eq!(LayerIds::surface(1), 1);
    assert!(!LayerIds::is_cave(2));
    assert_eq!(LayerIds::band(0), None);
    for (dim, band, id) in [(0, 5, 1055), (0, -4, 1046), (1, 0, 1150), (2, 49, 1299), (0, -50, 1000)] {
        assert_eq!(LayerIds::cave(dim, band), id);
        assert!(LayerIds::is_cave(id));
        assert_eq!((LayerIds::dim(id), LayerIds::band(id)), (dim, Some(band)));
    }
}

#[test]
fn bands_past_the_range_share_the_end_bands() {
    assert_eq!(LayerIds::cave(0, 400), LayerIds::cave(0, LayerIds::MAX_BAND));
    assert_eq!(LayerIds::cave(0, -400), LayerIds::cave(0, LayerIds::MIN_BAND));
    assert_eq!(LayerIds::dim(LayerIds::cave(1, 400)), 1);
}
