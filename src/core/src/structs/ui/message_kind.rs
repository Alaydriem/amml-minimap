// what a message line reports: something done for the player, or a note, refusal or error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Info,
    Done,
}
