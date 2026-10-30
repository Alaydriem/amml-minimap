use windows_sys::Win32::Foundation::{HWND, LPARAM};
use windows_sys::Win32::System::SystemInformation::GetTickCount64;
use windows_sys::Win32::System::Threading::GetCurrentProcessId;
use windows_sys::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowThreadProcessId, IsWindow,
};
use windows_sys::core::BOOL;

// the game's top-level window: the first one of this process with the class `Minecraft`; another game client has
// its own. A missing window is searched for again at most once a second
pub struct GameWindow {
    // the window handle as an integer, 0 for none, so the tracker can sit in the runtime's static
    hwnd: usize,
    next_search_ms: u64,
}

impl GameWindow {
    const SEARCH_EVERY_MS: u64 = 1000;
    const CLASS: [u16; 9] = [77, 105, 110, 101, 99, 114, 97, 102, 116];

    pub fn new() -> Self {
        Self { hwnd: 0, next_search_ms: 0 }
    }

    #[allow(unsafe_code)]
    pub fn hwnd(&mut self) -> Option<HWND> {
        // SAFETY: IsWindow takes any handle value
        if self.hwnd != 0 && unsafe { IsWindow(self.hwnd as HWND) } != 0 {
            return Some(self.hwnd as HWND);
        }
        self.hwnd = 0;
        // SAFETY: no arguments
        let now = unsafe { GetTickCount64() };
        if now < self.next_search_ms {
            return None;
        }
        self.next_search_ms = now + Self::SEARCH_EVERY_MS;
        let mut found: HWND = std::ptr::null_mut();
        // SAFETY: the callback writes one HWND through the LPARAM, which points to `found` for the call
        unsafe { EnumWindows(Some(Self::find_own_window), (&raw mut found) as LPARAM) };
        self.hwnd = found as usize;
        (!found.is_null()).then_some(found)
    }

    // the game window exists and has the focus
    #[allow(unsafe_code)]
    pub fn foreground(&mut self) -> bool {
        let Some(hwnd) = self.hwnd() else { return false };
        // SAFETY: no arguments
        unsafe { GetForegroundWindow() == hwnd }
    }

    #[allow(unsafe_code)]
    unsafe extern "system" fn find_own_window(hwnd: HWND, found: LPARAM) -> BOOL {
        let mut process = 0u32;
        // SAFETY: `process` is a live u32 for the call
        unsafe { GetWindowThreadProcessId(hwnd, &raw mut process) };
        // SAFETY: no arguments
        if process != unsafe { GetCurrentProcessId() } {
            return 1;
        }
        let mut name = [0u16; 16];
        // SAFETY: `name` holds 16 UTF-16 units, the count passed
        let len = unsafe { GetClassNameW(hwnd, name.as_mut_ptr(), 16) };
        if usize::try_from(len).ok() != Some(Self::CLASS.len()) || name[..Self::CLASS.len()] != Self::CLASS {
            return 1;
        }
        // SAFETY: `found` is the address of the HWND `hwnd()` passed to EnumWindows, live for the enumeration
        unsafe { *(found as *mut HWND) = hwnd };
        0
    }
}

impl Default for GameWindow {
    fn default() -> Self {
        Self::new()
    }
}
