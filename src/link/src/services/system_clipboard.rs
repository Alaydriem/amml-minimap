use crate::traits::ClipboardSink;

// the Windows clipboard; the manager does not offer one to mods
pub struct SystemClipboard;

impl ClipboardSink for SystemClipboard {
    fn write(&self, text: &str) -> Result<(), String> {
        clipboard_win::set_clipboard_string(text).map_err(|error| error.to_string())
    }
}
