use minimap_core::structs::settings::{SettingValue, SettingWrite};
use minimap_link::services::SettingsBridge;

use crate::support::{FakePort, PortCall};

#[test]
fn bridge_sends_ints_and_bools_as_hcl_values() {
    let port = FakePort::new();
    let writes = [
        SettingWrite { path: "mini_map.radius_chunks", value: SettingValue::Int(6) },
        SettingWrite { path: "world_map.show_chunk_grid", value: SettingValue::Bool(true) },
    ];

    SettingsBridge::flush(&port, &writes);

    assert_eq!(
        port.take_calls(),
        [
            PortCall::Setting("mini_map.radius_chunks".to_string(), hcl::Value::from(6)),
            PortCall::Setting("world_map.show_chunk_grid".to_string(), hcl::Value::from(true)),
        ]
    );
}

#[test]
fn a_refused_write_is_dropped_and_the_rest_still_go() {
    let mut port = FakePort::new();
    port.refuse_settings = true;
    let writes = [
        SettingWrite { path: "mini_map.radius_chunks", value: SettingValue::Int(6) },
        SettingWrite { path: "mini_map.show_coordinates", value: SettingValue::Bool(false) },
    ];

    SettingsBridge::flush(&port, &writes);
    SettingsBridge::flush(&port, &[]);

    assert_eq!(port.take_calls().len(), 2);
}
