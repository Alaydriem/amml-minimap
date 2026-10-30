use minimap_runtime::structs::CursorAction;
use windows_sys::Win32::Foundation::{POINT, RECT};
use windows_sys::Win32::Graphics::Gdi::ClientToScreen;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CURSORINFO, ClipCursor, GetClientRect, GetCursorInfo, GetForegroundWindow,
};

use crate::services::GameWindow;

// the system cursor: whether the game hides it, and the clip the scan plan asks for
pub struct CursorClip {
    // the last action applied was a clip, so a stop must give the cursor back
    clipped: bool,
}

impl CursorClip {
    pub fn new() -> Self {
        Self { clipped: false }
    }

    // gives back a clip this service made; nothing otherwise
    #[allow(unsafe_code)]
    pub fn release(&mut self) {
        if self.clipped {
            // SAFETY: a null rectangle releases the clip
            unsafe { ClipCursor(std::ptr::null()) };
            self.clipped = false;
        }
    }

    // the game has hidden the system cursor (it holds the mouse for camera control)
    #[allow(unsafe_code)]
    pub fn cursor_hidden(&self) -> bool {
        let mut cursor = CURSORINFO { cbSize: size_of::<CURSORINFO>() as u32, ..CURSORINFO::default() };
        // SAFETY: `cursor` is a live CURSORINFO whose cbSize names its size
        unsafe { GetCursorInfo(&raw mut cursor) != 0 && cursor.flags == 0 }
    }

    // clips the cursor to a 2 x 2 box at the client centre, or gives it back; only while the game has the focus
    #[allow(unsafe_code)]
    pub fn apply(&mut self, action: CursorAction, window: &mut GameWindow) {
        let Some(hwnd) = window.hwnd() else { return };
        // SAFETY: no arguments
        if unsafe { GetForegroundWindow() } != hwnd {
            return;
        }
        match action {
            CursorAction::None => {}
            CursorAction::ClipToCentre => {
                let mut client = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                // SAFETY: `client` is a live RECT for the call
                unsafe { GetClientRect(hwnd, &raw mut client) };
                let mut centre = POINT { x: (client.right - client.left) / 2, y: (client.bottom - client.top) / 2 };
                // SAFETY: `centre` is a live POINT for the call
                unsafe { ClientToScreen(hwnd, &raw mut centre) };
                let clip = RECT { left: centre.x - 1, top: centre.y - 1, right: centre.x + 1, bottom: centre.y + 1 };
                // SAFETY: `clip` is a live RECT for the call
                unsafe { ClipCursor(&raw const clip) };
                self.clipped = true;
            }
            CursorAction::Release => {
                // SAFETY: a null rectangle releases the clip
                unsafe { ClipCursor(std::ptr::null()) };
                self.clipped = false;
            }
        }
    }
}

impl Default for CursorClip {
    fn default() -> Self {
        Self::new()
    }
}
