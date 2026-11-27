use amml_api::{KeyCode, Modifiers};
use egui::{Event, Key, RawInput};
use minimap_core::config::KeysConfig;
use minimap_core::structs::input::{KeyPhase, KeybindAction};
use minimap_link::services::KeyFeed;

fn keys(raw: &RawInput) -> Vec<(Key, bool, bool)> {
    raw.events
        .iter()
        .filter_map(|event| match event {
            Event::Key { key, pressed, modifiers, .. } => Some((*key, *pressed, modifiers.ctrl)),
            _ => None,
        })
        .collect()
}

fn armed() -> KeyFeed {
    let feed = KeyFeed::new();
    feed.set_armed(true);
    feed
}

fn binds(feed: &KeyFeed) -> Vec<(KeybindAction, KeyPhase)> {
    let mut out = Vec::new();
    feed.take_binds(&mut out);
    out
}

#[test]
fn an_armed_escape_press_reaches_the_next_pass_once() {
    let feed = armed();

    assert!(feed.on_key(KeyCode::ESCAPE, true, false, Modifiers::NONE, true));
    let mut first = RawInput::default();
    feed.drain_into(&mut first);
    let mut second = RawInput::default();
    feed.drain_into(&mut second);

    assert_eq!(keys(&first), vec![(Key::Escape, true, false), (Key::Escape, false, false)]);
    assert!(second.events.is_empty());
}

#[test]
fn ctrl_z_reaches_the_pass_as_an_undo_press() {
    let feed = armed();

    feed.on_key(KeyCode::Z, true, false, Modifiers::CTRL, true);
    let mut raw = RawInput::default();
    feed.drain_into(&mut raw);

    assert_eq!(keys(&raw), vec![(Key::Z, true, true), (Key::Z, false, true)]);
}

#[test]
fn repeats_and_unbound_keys_are_not_taken() {
    let feed = armed();

    let taken = [
        feed.on_key(KeyCode::ESCAPE, true, true, Modifiers::NONE, true),
        feed.on_key(KeyCode::ESCAPE, false, false, Modifiers::NONE, true),
        feed.on_key(KeyCode(0x41), true, false, Modifiers::NONE, true),
        feed.on_key(KeyCode::Z, true, false, Modifiers::NONE, true),
    ];
    let mut raw = RawInput::default();
    feed.drain_into(&mut raw);

    assert_eq!(taken, [false; 4]);
    assert!(raw.events.is_empty());
    assert!(binds(&feed).is_empty());
}

#[test]
fn a_feed_that_is_not_armed_takes_nothing_and_disarming_drops_waiting_presses() {
    let feed = KeyFeed::new();

    assert!(!feed.on_key(KeyCode::ESCAPE, true, false, Modifiers::NONE, true));
    assert!(!feed.on_key(KeyCode::M, true, false, Modifiers::NONE, true));
    feed.set_armed(true);
    feed.on_key(KeyCode::ESCAPE, true, false, Modifiers::NONE, true);
    feed.set_armed(false);
    let mut raw = RawInput::default();
    feed.drain_into(&mut raw);

    assert!(raw.events.is_empty());
    assert!(binds(&feed).is_empty());
}

#[test]
fn a_bind_key_gives_a_press_then_a_release_of_its_action() {
    let feed = armed();

    assert!(feed.on_key(KeyCode::M, true, false, Modifiers::NONE, true));
    feed.on_key(KeyCode::M, true, true, Modifiers::NONE, true);
    feed.on_key(KeyCode::M, false, false, Modifiers::NONE, true);

    assert_eq!(
        binds(&feed),
        vec![(KeybindAction::ToggleWorldMap, KeyPhase::Pressed), (KeybindAction::ToggleWorldMap, KeyPhase::Released)]
    );
}

#[test]
fn the_chords_are_the_configured_ones_with_their_modifiers() {
    let feed = armed();
    feed.set_keys(&KeysConfig { toggle_world_map: "Ctrl+Shift+M".to_string(), ..KeysConfig::default() });

    feed.on_key(KeyCode::M, true, false, Modifiers::NONE, true);
    feed.on_key(KeyCode::M, false, false, Modifiers::NONE, true);
    feed.on_key(KeyCode::M, true, false, Modifiers::CTRL | Modifiers::SHIFT, true);
    feed.on_key(KeyCode::M, true, false, Modifiers::F3, true);
    feed.on_key(KeyCode::EQUALS, true, false, Modifiers::NONE, true);

    assert_eq!(
        binds(&feed),
        vec![
            (KeybindAction::ToggleWorldMap, KeyPhase::Pressed),
            (KeybindAction::ToggleMinimap, KeyPhase::Pressed),
            (KeybindAction::ZoomIn, KeyPhase::Pressed),
        ]
    );
}

#[test]
fn disarming_releases_a_held_bind() {
    let feed = armed();
    feed.on_key(KeyCode::MINUS, true, false, Modifiers::NONE, true);

    feed.set_armed(false);

    assert_eq!(
        binds(&feed),
        vec![(KeybindAction::ZoomOut, KeyPhase::Pressed), (KeybindAction::ZoomOut, KeyPhase::Released)]
    );
}

#[test]
fn a_bind_press_the_lease_did_not_block_is_left_to_the_manager() {
    let feed = armed();

    let taken = feed.on_key(KeyCode::M, true, false, Modifiers::NONE, false);

    assert!(!taken);
    assert!(binds(&feed).is_empty());
}

fn events(feed: &KeyFeed) -> Vec<Event> {
    let mut raw = RawInput::default();
    feed.drain_into(&mut raw);
    raw.events
}

#[test]
fn while_a_field_has_the_keyboard_the_blocked_editing_keys_reach_the_pass() {
    let feed = armed();
    feed.set_typing(true);

    for key in [KeyCode(0x08), KeyCode(0x25), KeyCode(0x0D)] {
        feed.on_key(key, true, false, Modifiers::NONE, true);
    }
    feed.on_key(KeyCode(0x08), true, true, Modifiers::NONE, true);
    feed.on_key(KeyCode(0x27), true, false, Modifiers::SHIFT, true);

    let pressed: Vec<(Key, bool)> = events(&feed)
        .iter()
        .filter_map(|event| match event {
            Event::Key { key, pressed: true, modifiers, .. } => Some((*key, modifiers.shift)),
            _ => None,
        })
        .collect();
    assert_eq!(
        pressed,
        vec![
            (Key::Backspace, false),
            (Key::ArrowLeft, false),
            (Key::Enter, false),
            (Key::Backspace, false),
            (Key::ArrowRight, true)
        ]
    );
}

#[test]
fn while_typing_ctrl_c_and_ctrl_x_copy_and_cut_and_letters_match_no_bind() {
    let feed = armed();
    feed.set_typing(true);

    feed.on_key(KeyCode(0x43), true, false, Modifiers::CTRL, true);
    feed.on_key(KeyCode(0x58), true, false, Modifiers::CTRL, true);
    feed.on_key(KeyCode::M, true, false, Modifiers::NONE, true);
    let out = events(&feed);

    assert!(out.contains(&Event::Copy));
    assert!(out.contains(&Event::Cut));
    assert!(binds(&feed).is_empty());
}

#[test]
fn editing_keys_are_not_fed_when_no_field_has_the_keyboard_or_the_lease_did_not_block_them() {
    let feed = armed();
    feed.on_key(KeyCode(0x08), true, false, Modifiers::NONE, true);
    feed.set_typing(true);
    feed.on_key(KeyCode(0x08), true, false, Modifiers::NONE, false);

    assert!(events(&feed).is_empty());
}

#[test]
fn escape_and_ctrl_z_are_counted_only_when_the_lease_blocked_them() {
    let feed = armed();

    let escape = feed.on_key(KeyCode::ESCAPE, true, false, Modifiers::NONE, false);
    let undo = feed.on_key(KeyCode::Z, true, false, Modifiers::CTRL, false);

    assert!(!escape && !undo);
    assert!(events(&feed).is_empty());
}

#[test]
fn while_typing_ctrl_z_is_a_text_undo_of_the_field_and_not_a_map_undo() {
    let feed = armed();
    feed.set_typing(true);

    feed.on_key(KeyCode::Z, true, false, Modifiers::CTRL, true);
    feed.on_key(KeyCode::Z, true, true, Modifiers::CTRL, true);

    let presses = events(&feed)
        .iter()
        .filter(|event| matches!(event, Event::Key { key: Key::Z, pressed: true, modifiers, .. } if modifiers.ctrl))
        .count();
    assert_eq!(presses, 2, "one per press, repeats included, as an editing key");
}
