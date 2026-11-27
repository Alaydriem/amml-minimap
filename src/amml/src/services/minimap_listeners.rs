use std::sync::Arc;

use amml_api::amml_abi::{BlockChanged, ClientTick, KeyInput, WorldLeft};
use amml_api::{ApiError, BlockChangedExt, Ctx, HookOrder, Listener};

use crate::services::MinimapRuntime;
use crate::types::QueuedColumn;

// the loader listeners of the minimap; each only hands its event to the runtime. Dropping them removes them
pub struct MinimapListeners {
    listeners: Vec<Listener>,
    runtime: Arc<MinimapRuntime>,
}

impl MinimapListeners {
    pub fn start(ctx: &Ctx, runtime: Arc<MinimapRuntime>) -> Result<Self, ApiError> {
        let events = ctx.events();
        let mut listeners = Vec::with_capacity(4);
        let ticking = Arc::clone(&runtime);
        listeners.push(events.listen::<ClientTick>(HookOrder::NORMAL, move |_, at| {
            if let Some(client) = at.client() {
                ticking.tick(&client);
            }
        })?);
        let leaving = Arc::clone(&runtime);
        listeners.push(events.listen::<WorldLeft>(HookOrder::NORMAL, move |_, _| leaving.world_left())?);
        let changing = Arc::clone(&runtime);
        listeners.push(events.listen::<BlockChanged>(HookOrder::NORMAL, move |event, _| {
            // a change inside one block type keeps the map colour
            if event.new() == event.old() {
                return;
            }
            let pos = event.pos();
            changing.block_changed(QueuedColumn { dimension: event.dimension(), x: pos.x, z: pos.z });
        })?);
        let typing = Arc::clone(&runtime);
        listeners.push(events.listen::<KeyInput>(HookOrder::NORMAL, move |event, _| typing.key(event))?);
        ctx.log().info("minimap block changes on", &[]);
        Ok(Self { listeners, runtime })
    }

    // the listeners go first, so no listener runs while the runtime stops
    pub fn stop(self) {
        let Self { listeners, runtime } = self;
        drop(listeners);
        runtime.stop();
    }
}
