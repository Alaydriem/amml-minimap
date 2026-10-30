use std::sync::Arc;

use amml_i18n::localize::Localizer;
use amml_i18n::message::Message;
use amml_i18n::service::{LocaleSetting, LocalizationService};
use amml_ui::LeviUiMessage;
use amml_ui::text::LeviUiPacks;

use crate::message::MinimapMessage;
use crate::text::MinimapPacks;

/// The minimap's words in one language: its own packs and the amml-ui widgets' packs.
pub struct MinimapLocale;

impl MinimapLocale {
    /// `code` is the manager's language code: empty for English, `en_XA` for the pseudo-locale.
    pub fn localizer(code: &str) -> Arc<Localizer> {
        let service = LocalizationService::new(vec![
            (LeviUiMessage::DOMAIN, LeviUiPacks::SOURCE),
            (MinimapMessage::DOMAIN, MinimapPacks::SOURCE),
        ]);
        Arc::new(service.load(&LocaleSetting::from_code(code)))
    }
}
