use std::collections::HashMap;

use amml_game::{EntryId, EntryResolver};
use amml_sdk::SdkError;

// a resolver that knows the ids it was given
pub struct FakeResolver {
    known: HashMap<&'static str, u64>,
}

impl FakeResolver {
    pub fn new(values: &[(EntryId, u64)]) -> Self {
        Self { known: values.iter().map(|(id, value)| (id.as_str(), *value)).collect() }
    }
}

impl EntryResolver for FakeResolver {
    fn resolve(&self, id: &str) -> Result<u64, SdkError> {
        self.known.get(id).copied().ok_or_else(|| SdkError::MissingEntry(id.to_string()))
    }
}
