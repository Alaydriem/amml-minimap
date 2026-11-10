use std::cell::RefCell;
use std::rc::Rc;

use minimap_link::traits::ClipboardSink;

// keeps what the session copied instead of writing the Windows clipboard
#[derive(Clone, Default)]
pub struct FakeClipboard {
    pub copied: Rc<RefCell<Vec<String>>>,
}

impl ClipboardSink for FakeClipboard {
    fn write(&self, text: &str) -> Result<(), String> {
        self.copied.borrow_mut().push(text.to_string());
        Ok(())
    }
}
