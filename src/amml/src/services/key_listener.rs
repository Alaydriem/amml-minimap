use amml_api::amml_abi::KeyInput;
use amml_api::{Event, KeyInputExt};
use minimap_link::services::KeyFeed;

// hands a real key event of the loader to the key feed of the world map; a key a mod injected is not the user's.
// `f3`: F3 is held (from `Input::modifiers`; a `KeyInput` never carries it)
pub struct KeyListener;

impl KeyListener {
    pub fn on_key(feed: &KeyFeed, event: &KeyInput, f3: bool) -> bool {
        if event.is_injected() {
            return false;
        }
        let modifiers = event.modifiers().with_f3(f3);
        feed.on_key(event.key_code(), event.down != 0, event.repeat != 0, modifiers, event.is_cancelled())
    }
}
