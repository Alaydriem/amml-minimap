#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyPhase {
    Pressed,
    Released,
    // the key is still down and the system repeats it
    Repeat,
}
