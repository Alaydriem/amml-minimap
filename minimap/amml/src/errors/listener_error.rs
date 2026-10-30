use amml_msvc::MsvcError;

#[derive(Debug, thiserror::Error)]
pub enum ListenerError {
    #[error("listener slot {slot} is outside the 9-slot table")]
    SlotOutOfRange { slot: u64 },
    #[error("listener object not allocated: {0}")]
    Allocation(#[from] MsvcError),
    #[error("the BlockSourceListener table is null")]
    NoBaseTable,
}
