use amml_ui::id::AmmlUiId;
use egui::Id;
use minimap_ui::id::MinimapUiId;

/// An id enum from another crate, with a derived `Hash`.
#[derive(Debug, Clone, Copy, Hash)]
enum OtherId {
    First,
    Second,
}

#[test]
fn a_minimap_ui_id_never_equals_the_same_numbered_variant_of_another_enum() {
    assert_ne!(Id::unique(MinimapUiId::Map), Id::unique(OtherId::First));
    assert_ne!(Id::unique(MinimapUiId::Tools), Id::unique(OtherId::Second));
    assert_ne!(Id::unique(MinimapUiId::Map), Id::unique(AmmlUiId::Locale));
    assert_ne!(Id::unique(MinimapUiId::Tools), Id::unique(AmmlUiId::SegmentSlide));
}

#[test]
fn minimap_ui_id_variants_keep_their_own_ids() {
    assert_ne!(MinimapUiId::Map.id(), MinimapUiId::Tools.id());
}
