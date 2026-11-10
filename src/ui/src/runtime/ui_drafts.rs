// text being typed into the UI's fields; it lives next to the egui text fields and reaches core with
// the action that uses it. Both strings keep their capacity
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct UiDrafts {
    pub search: String,
    pub name: String,
    // the dialog the name belongs to; a new dialog starts from its own name
    pub dialog_serial: u64,
}

impl UiDrafts {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.search.clear();
        self.name.clear();
        self.dialog_serial = 0;
    }
}
