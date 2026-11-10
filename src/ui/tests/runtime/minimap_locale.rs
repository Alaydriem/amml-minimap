use amml_i18n::message::Message;
use amml_i18n::text::Text;
use minimap_ui::message::MinimapMessage;
use minimap_ui::runtime::MinimapLocale;

#[test]
fn pseudo_locale_changes_every_word() {
    let localizer = MinimapLocale::localizer("en_XA");
    for message in MinimapMessage::ALL.iter().filter(|message| !message.msgid().contains('{')) {
        let resolved = Text::message(*message).resolve(&localizer).into_owned();
        assert_ne!(resolved, message.msgid(), "{message:?} has no pseudo form");
    }
}

#[test]
fn empty_code_is_english() {
    let localizer = MinimapLocale::localizer("");
    assert_eq!(Text::message(MinimapMessage::Save).resolve(&localizer), "Save");
}
