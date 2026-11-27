use std::sync::{Mutex, PoisonError};

use amml_api::amml_abi::{
    BlockChanged, ClientTick, InjectedInputReleased, InputLeaseLost, InputLeaseResumed, InputLeaseSuspended, KeyInput,
    ModsLoaded, ProviderGone, ProviderReady, ScreenshotTaken, ScreenshotWritten, WorldLeft,
};
use amml_api::{AmmlMod, ApiError, Area, Ctx, DisableCtx, Uses};

use crate::services::{MinimapListeners, MinimapRuntime};

// what the enabled mod owns. `disable` takes it out of the lock, then drops the listeners and stops the runtime
static RUNNING: Mutex<Option<MinimapListeners>> = Mutex::new(None);

// the AMML entry point: enable starts the manager link and the listeners, disable removes them, gives the cursor
// back and leaves the mod manager
pub struct MinimapMod;

impl AmmlMod for MinimapMod {
    const NAMESPACE: &'static str = "alaydriem";
    const NAME: &'static str = "minimap";
    const USES: Uses = amml_api::amml_uses! {
        feature "minimap" {
            required ClientTick;
            required WorldLeft;
            required KeyInput;
            required ProviderReady;
            required ProviderGone;
            required ModsLoaded;
            required InputLeaseSuspended;
            required InputLeaseResumed;
            required InputLeaseLost;
            required InjectedInputReleased;
            required ScreenshotTaken;
            required ScreenshotWritten;
            required area Area::INPUT_LEASE;
            required area Area::CLIENT_READ;
            required area Area::CLIENT_MOUSE;
            required area Area::WINDOW_READ;
            required area Area::ACTOR_READ.at(3);
            required area Area::WORLD_CHUNK_READ;
        }
        feature "world_identity" {
            required area Area::CLIENT_WORLD_INFO;
        }
        feature "biomes" {
            required area Area::WORLD_BIOMES;
        }
        feature "block_changes" {
            required BlockChanged;
        }
        feature "vanilla_colors" {
            required area Area::WORLD_MAP_COLOR;
        }
        feature "cursor_clip" {
            required area Area::WINDOW_CURSOR_CLIP;
        }
    };

    fn enable(ctx: &Ctx) -> Result<(), ApiError> {
        let log = ctx.log();
        let Some(data_dir) = ctx.dirs().data_dir() else {
            log.error("minimap off", &[("error", "the loader gives no data folder")]);
            return Err(ApiError::Refused);
        };
        let runtime = MinimapRuntime::start(ctx, &data_dir).map_err(|error| {
            log.error("minimap off", &[("error", &error.to_string())]);
            ApiError::Refused
        })?;
        let listeners = MinimapListeners::start(ctx, runtime).inspect_err(|error| {
            log.error("minimap off", &[("error", &error.to_string())]);
        })?;
        *RUNNING.lock().unwrap_or_else(PoisonError::into_inner) = Some(listeners);
        Ok(())
    }

    fn disable(_ctx: &Ctx, _how: &DisableCtx) {
        let running = RUNNING.lock().unwrap_or_else(PoisonError::into_inner).take();
        if let Some(listeners) = running {
            listeners.stop();
        }
    }
}
