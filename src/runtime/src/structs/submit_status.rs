#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitStatus {
    Loaded,
    NotLoaded,
    TopPlaceholder,
    NotProbed,
}

impl SubmitStatus {
    pub fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Loaded),
            1 => Some(Self::NotLoaded),
            2 => Some(Self::TopPlaceholder),
            3 => Some(Self::NotProbed),
            _ => None,
        }
    }
}
