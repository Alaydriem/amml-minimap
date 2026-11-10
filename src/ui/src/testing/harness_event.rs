// one input event as the game window gives it; `code` is a Windows virtual key, a wheel turn is 120 per notch
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessEvent {
    Text(char),
    Key { code: i32, down: bool, ctrl: bool, shift: bool },
    Wheel(i32),
}
