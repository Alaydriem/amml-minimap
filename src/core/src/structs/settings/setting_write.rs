use crate::structs::settings::SettingValue;

// a value the player changed in play that is saved as a manager setting at `path`
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettingWrite {
    pub path: &'static str,
    pub value: SettingValue,
}
