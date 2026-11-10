// what the host does with the Win32 cursor clip this tick
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorAction {
    #[default]
    None,
    // the game hides the cursor: keep it inside a 2 x 2 box at the client centre
    ClipToCentre,
    Release,
}
