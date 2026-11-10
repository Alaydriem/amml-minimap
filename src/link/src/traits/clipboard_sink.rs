// where text a menu choice copied goes; the Windows clipboard in the mod, a list in tests
pub trait ClipboardSink {
    // Err is a reason to log
    fn write(&self, text: &str) -> Result<(), String>;
}
