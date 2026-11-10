use minimap_core::structs::settings::{SettingValue, SettingWrite};

use crate::traits::ManagerPort;

// saves the in-play changes as manager settings; the manager merges the writes, so a refused one is logged
// and dropped, never retried
pub struct SettingsBridge;

impl SettingsBridge {
    pub fn flush<P: ManagerPort>(port: &P, writes: &[SettingWrite]) {
        for write in writes {
            let value = match write.value {
                SettingValue::Bool(on) => hcl::Value::from(on),
                SettingValue::Int(number) => hcl::Value::from(number),
            };
            if let Err(error) = port.set_setting(write.path, &value) {
                curia::warn!("setting not saved", { path: write.path, error: error.to_string() });
            }
        }
    }
}
