use crate::structs::chunk::ChangeKinds;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SubmitResult {
    pub changes: ChangeKinds,
    pub first_load: bool,
    pub previous_range: Option<(i16, i16)>,
    pub new_range: Option<(i16, i16)>,
    pub needs_retry: bool,
}
