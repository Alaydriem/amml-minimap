use amm_client::test_support::FakeTable;
use amm_core::structs::event::ModEvent;
use minimap_link::traits::ManagerPort;

#[test]
fn drain_events_takes_more_than_one_batch() {
    let mut link = FakeTable::link_without_frame();
    for _ in 0..150 {
        FakeTable::push_event(ModEvent::WorldReloadApply);
    }
    let mut events = Vec::new();

    link.drain_events(&mut events);

    assert_eq!(events.len(), 150);
}

#[test]
fn config_reads_the_manager_document_and_clamps_it() {
    let link = FakeTable::link_without_frame();
    FakeTable::set_config("mini_map {\n  radius_chunks = 60\n  size_ratio = 0.3\n}\n");

    let config = match ManagerPort::config(&link) {
        Ok(config) => config,
        Err(error) => panic!("config not read: {error}"),
    };

    assert_eq!(config.mini_map.radius_chunks, 16);
    assert_eq!(config.mini_map.size_ratio, 0.3);
}

#[test]
fn texture_budget_reads_the_manager_grant() {
    let link = FakeTable::link_without_frame();
    FakeTable::set_texture_grant(amm_core::structs::manager::TextureGrant { bytes: 300 * 1024 * 1024, count: 1500 });

    let grant = link.texture_budget_grant();

    assert_eq!(grant, minimap_core::structs::draw::TextureGrant { bytes: 300 * 1024 * 1024, count: 1500 });
}
