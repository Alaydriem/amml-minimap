use alaydriem_minimap::services::KeyListener;
use amml_api::amml_abi::{AmmlModifiers, KeyInput};
use amml_api::testing::FakeGame;
use egui::RawInput;
use minimap_core::structs::input::{KeyPhase, KeybindAction};
use minimap_link::services::KeyFeed;

fn key(vk: u16, down: bool, repeat: bool, modifiers: AmmlModifiers, injected: bool) -> KeyInput {
    let mut header = FakeGame::header::<KeyInput>();
    header.injected = u8::from(injected);
    // the input lease of the open map blocks the keys
    header.cancelled = 1;
    KeyInput { header, key: vk, down: u8::from(down), repeat: u8::from(repeat), modifiers }
}

fn fed(feed: &KeyFeed) -> usize {
    let mut raw = RawInput::default();
    feed.drain_into(&mut raw);
    raw.events.len()
}

#[test]
fn a_real_escape_press_is_fed_while_armed() {
    let feed = KeyFeed::new();
    feed.set_armed(true);

    KeyListener::on_key(&feed, &key(0x1B, true, false, AmmlModifiers(0), false), false);

    assert_eq!(fed(&feed), 2);
}

#[test]
fn a_ctrl_z_press_with_either_ctrl_is_fed() {
    let feed = KeyFeed::new();
    feed.set_armed(true);

    KeyListener::on_key(&feed, &key(0x5A, true, false, AmmlModifiers::LCTRL, false), false);

    assert_eq!(fed(&feed), 2);
}

#[test]
fn an_injected_key_and_a_repeat_are_not_fed() {
    let feed = KeyFeed::new();
    feed.set_armed(true);

    KeyListener::on_key(&feed, &key(0x1B, true, false, AmmlModifiers(0), true), false);
    KeyListener::on_key(&feed, &key(0x1B, true, true, AmmlModifiers(0), false), false);

    assert_eq!(fed(&feed), 0);
}

#[test]
fn f3_from_the_input_service_picks_the_f3_bind() {
    let feed = KeyFeed::new();
    feed.set_armed(true);

    KeyListener::on_key(&feed, &key(0x4D, true, false, AmmlModifiers(0), false), true);
    let mut binds = Vec::new();
    feed.take_binds(&mut binds);

    assert_eq!(binds, vec![(KeybindAction::ToggleMinimap, KeyPhase::Pressed)]);
}

#[test]
fn a_bind_press_no_lease_blocked_is_not_matched() {
    let feed = KeyFeed::new();
    feed.set_armed(true);
    let mut event = key(0x4D, true, false, AmmlModifiers(0), false);
    event.header.cancelled = 0;

    KeyListener::on_key(&feed, &event, false);
    let mut binds = Vec::new();
    feed.take_binds(&mut binds);

    assert!(binds.is_empty());
}
