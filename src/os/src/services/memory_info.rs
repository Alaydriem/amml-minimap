use windows_sys::Win32::System::ProcessStatus::{
    K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS, PROCESS_MEMORY_COUNTERS_EX,
};
use windows_sys::Win32::System::Threading::GetCurrentProcess;

// process memory for telemetry, read about once a second at 20 ticks a second; the call takes microseconds
pub struct MemoryInfo {
    ticks: u32,
    working_set: u64,
    private_bytes: u64,
}

impl MemoryInfo {
    const EVERY_TICKS: u32 = 20;

    pub fn new() -> Self {
        Self { ticks: 0, working_set: 0, private_bytes: 0 }
    }

    // the working set and private bytes, read again every EVERY_TICKS calls
    pub fn sample(&mut self) -> (u64, u64) {
        let due = self.ticks.is_multiple_of(Self::EVERY_TICKS);
        self.ticks = self.ticks.wrapping_add(1);
        if due {
            self.read();
        }
        (self.working_set, self.private_bytes)
    }

    #[allow(unsafe_code)]
    fn read(&mut self) {
        let size = size_of::<PROCESS_MEMORY_COUNTERS_EX>() as u32;
        let mut counters = PROCESS_MEMORY_COUNTERS_EX { cb: size, ..PROCESS_MEMORY_COUNTERS_EX::default() };
        // SAFETY: `counters` is a live PROCESS_MEMORY_COUNTERS_EX whose `cb` names its size; the EX layout starts
        // with the plain counters, so the call may write it through that pointer type
        let read = unsafe {
            K32GetProcessMemoryInfo(GetCurrentProcess(), (&raw mut counters).cast::<PROCESS_MEMORY_COUNTERS>(), size)
        };
        if read != 0 {
            self.working_set = counters.WorkingSetSize as u64;
            self.private_bytes = counters.PrivateUsage as u64;
        }
    }
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self::new()
    }
}
