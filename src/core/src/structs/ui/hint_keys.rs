// the keybind chords the world map's hint bar names, as the manager writes them
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HintKeys<'a> {
    pub toggle: &'a str,
}
