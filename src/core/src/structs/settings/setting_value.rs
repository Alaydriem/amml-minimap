#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingValue {
    Bool(bool),
    Int(i64),
}
