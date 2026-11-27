// where the colour of a shown block came from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSource {
    // the scan plan's colour table
    Table,
    // the game's own map colour
    Vanilla,
    // a fixed colour per kind of block
    Fallback,
    // nothing is shown, or the column is void
    None,
}

impl ColorSource {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Vanilla => "vanilla",
            Self::Fallback => "fallback",
            Self::None => "none",
        }
    }
}
