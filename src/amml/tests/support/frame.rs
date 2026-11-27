use std::sync::{Arc, Mutex};

use amml_api::amml_abi::ClientTick;
use amml_api::testing::FakeGame;
use amml_api::{Client, HookOrder};

use crate::support::MinimapRig;

// runs a closure inside one client tick of a fake game, with the tick's client, and gives back what it returns
pub struct Frame;

impl Frame {
    pub fn run<R: Send + 'static>(game: &FakeGame, f: impl FnOnce(&Client<'_>) -> R + Send + 'static) -> R {
        let body = Arc::new(Mutex::new(Some(f)));
        let result: Arc<Mutex<Option<R>>> = Arc::new(Mutex::new(None));
        let (taken, out) = (Arc::clone(&body), Arc::clone(&result));
        let listener = MinimapRig::ctx(game)
            .events()
            .listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
                let Some(f) = taken.lock().unwrap().take() else { return };
                let client = at.client().unwrap();
                *out.lock().unwrap() = Some(f(&client));
            })
            .unwrap();
        MinimapRig::tick(game);
        drop(listener);
        let value = result.lock().unwrap().take();
        value.unwrap()
    }
}
