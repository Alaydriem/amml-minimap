use windows_sys::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, GetLastError, STILL_ACTIVE};
use windows_sys::Win32::System::Threading::{GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION};

// whether a process id names a running process. A process that cannot be asked (another user's, or an error) counts
// as running, so a caller that deletes on "not running" never deletes in doubt
pub struct ProcessProbe;

impl ProcessProbe {
    #[allow(unsafe_code)]
    pub fn is_running(pid: u32) -> bool {
        if pid == std::process::id() {
            return true;
        }
        // SAFETY: OpenProcess takes any id and returns null when it cannot open one
        let process = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid) };
        if process.is_null() {
            // SAFETY: no arguments; reads this thread's last error, set by OpenProcess
            return unsafe { GetLastError() } != ERROR_INVALID_PARAMETER;
        }
        let mut code = 0u32;
        // SAFETY: `process` is an open handle with query rights and `code` a live u32 for the call
        let read = unsafe { GetExitCodeProcess(process, &raw mut code) };
        // SAFETY: the handle was opened above and is closed once
        unsafe { CloseHandle(process) };
        read == 0 || code == STILL_ACTIVE as u32
    }
}
