use amml_api::{ApiError, CursorClip as Clip, Logger, Window};
use minimap_runtime::structs::CursorAction;

// the cursor clip the scan plan asks for, through the loader's window. The clip is dropped while the game does not
// hold the mouse (menus, chat, the world map), at a world leave and at disable. The loader drops it at focus loss
// and applies it again when the focus returns, so a focus loss changes only what is logged. No clip is asked while
// the loader knows no game window. A refused request is warned once per error kind
pub struct CursorClip {
    window: Window,
    log: Logger,
    // the loader holds a clip request of this mod
    requested: bool,
    // the clip holds the cursor now: requested and the window in the foreground
    holding: bool,
    // a bit per ApiError kind already warned about
    warned: u8,
}

impl CursorClip {
    pub fn new(window: Window, log: Logger) -> Self {
        Self { window, log, requested: false, holding: false, warned: 0 }
    }

    // once per client tick: `mouse_captured`, the game holds the mouse; `foreground`, the game window is in front
    // (`None`: the loader knows no game window yet)
    pub fn apply(&mut self, action: CursorAction, mouse_captured: bool, foreground: Option<bool>) {
        let window = foreground.is_some();
        let foreground = foreground.unwrap_or(false);
        let wanted = match action {
            CursorAction::ClipToCentre => true,
            CursorAction::Release => false,
            CursorAction::None => self.requested,
        };
        let wanted = wanted && mouse_captured && window;
        if wanted != self.requested {
            self.request(wanted);
        }
        let reason = if !foreground {
            "focus"
        } else if !mouse_captured {
            "mouse_free"
        } else {
            "released"
        };
        self.show(self.requested && foreground, reason);
    }

    // gives the cursor back now; `reason` names why in the log
    pub fn drop_clip(&mut self, reason: &'static str) {
        if self.requested {
            self.request(false);
        }
        self.show(false, reason);
    }

    pub fn holding(&self) -> bool {
        self.holding
    }

    fn request(&mut self, clip: bool) {
        let asked = if clip { Some(Clip::Centre) } else { None };
        match self.window.clip_cursor(asked) {
            Ok(()) => self.requested = clip,
            Err(error) => {
                self.warn(&error);
                if !clip {
                    self.requested = false;
                }
            }
        }
    }

    fn warn(&mut self, error: &ApiError) {
        let bit = match error {
            ApiError::NotAvailable => 1,
            ApiError::NotReady => 2,
            ApiError::Refused => 4,
            ApiError::Stale => 8,
            _ => 16,
        };
        if self.warned & bit == 0 {
            self.warned |= bit;
            self.log.warn("minimap cursor clip refused", &[("error", &error.to_string())]);
        }
    }

    fn show(&mut self, holding: bool, reason: &'static str) {
        if holding == self.holding {
            return;
        }
        self.holding = holding;
        if holding {
            self.log.info("minimap cursor clip on", &[]);
        } else {
            self.log.info("minimap cursor clip off", &[("reason", reason)]);
        }
    }
}
