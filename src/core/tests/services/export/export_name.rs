use minimap_core::services::export::ExportName;
use minimap_core::structs::export::ExportKind;

#[test]
fn a_name_holds_the_world_dimension_utc_time_and_kind() {
    assert_eq!(
        ExportName::file("My World", 0, 1_792_138_360, ExportKind::View),
        "My_World-overworld-2026-10-16_08-12-40-view.png"
    );
    assert_eq!(ExportName::file("a:b", 1, 0, ExportKind::Area), "a_b-nether-1970-01-01_00-00-00-area.png");
}

#[test]
fn leap_days_and_unknown_dimensions_are_named_correctly() {
    assert_eq!(ExportName::file("w", 2, 951_786_061, ExportKind::Area), "w-end-2000-02-29_01-01-01-area.png");
    assert_eq!(ExportName::dimension(7), "dim7");
    assert_eq!(ExportName::file("", -1, 0, ExportKind::View), "world-dim-1-1970-01-01_00-00-00-view.png");
}

#[test]
fn a_cave_layer_is_named_by_its_dimension_and_band() {
    use minimap_core::services::cave::LayerIds;
    assert_eq!(ExportName::dimension(LayerIds::cave(0, 5)), "overworld-cave-5");
    assert_eq!(ExportName::dimension(LayerIds::cave(1, -2)), "nether-cave--2");
}
