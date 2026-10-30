use amml_i18n::service::PackSource;

/// The compiled language packs of `MinimapMessage`, built into the minimap DLL. Each file in `i18n/packs/` is
/// listed here; a test fails when one is missing.
pub struct MinimapPacks;

impl MinimapPacks {
    pub const SOURCE: PackSource = PackSource::Embedded(&[("en_XA", include_bytes!("../../i18n/packs/en_XA.json"))]);
}
