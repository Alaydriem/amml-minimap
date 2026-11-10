use amm_core::structs::descriptor::Capability;
use amm_core::structs::event::ModEvent;
use amm_core::structs::lease::AcquireOutcome;
use amm_core::structs::registry::ModIndex;
use minimap_core::structs::view::ViewFlags;
use minimap_link::services::LeaseController;
use minimap_link::structs::LeaseEffect;

use crate::support::{FakePort, PortCall};

const CLOSED: ViewFlags = ViewFlags { world_map_open: false, show_minimap: true };
const OPEN: ViewFlags = ViewFlags { world_map_open: true, show_minimap: true };

// a controller that already holds Hud, and a port with no calls recorded
fn started(port: &FakePort) -> LeaseController {
    let mut leases = LeaseController::new();
    leases.sync(port, CLOSED);
    port.take_calls();
    leases
}

#[test]
fn the_first_sync_takes_the_hud_lease() {
    let port = FakePort::new();
    let mut leases = LeaseController::new();

    leases.sync(&port, CLOSED);

    assert_eq!(port.take_calls(), [PortCall::Acquire(Capability::Hud)]);
}

#[test]
fn opening_acquires_overlay_then_input_then_cursor() {
    let port = FakePort::new();
    let mut leases = started(&port);

    let effect = leases.sync(&port, OPEN);

    assert_eq!(effect, None);
    assert_eq!(
        port.take_calls(),
        [PortCall::Acquire(Capability::Overlay), PortCall::Acquire(Capability::Input), PortCall::Cursor(true)]
    );
}

#[test]
fn closing_turns_the_cursor_off_before_releasing() {
    let port = FakePort::new();
    let mut leases = started(&port);
    leases.sync(&port, OPEN);
    port.take_calls();

    leases.sync(&port, CLOSED);

    assert_eq!(
        port.take_calls(),
        [PortCall::Cursor(false), PortCall::Release(Capability::Input), PortCall::Release(Capability::Overlay)]
    );
}

#[test]
fn denied_overlay_closes_the_map_and_takes_nothing() {
    let mut port = FakePort::new();
    let mut leases = started(&port);
    port.outcomes.insert(Capability::Overlay, AcquireOutcome::Denied { holder: ModIndex(3) });

    let effect = leases.sync(&port, OPEN);

    assert_eq!(effect, Some(LeaseEffect::CloseWorldMap));
    assert_eq!(port.take_calls(), [PortCall::Acquire(Capability::Overlay), PortCall::Holder(Capability::Overlay)]);
    leases.sync(&port, CLOSED);
    assert_eq!(port.take_calls(), []);
}

#[test]
fn denied_input_releases_overlay_and_closes_the_map() {
    let mut port = FakePort::new();
    let mut leases = started(&port);
    port.outcomes.insert(Capability::Input, AcquireOutcome::Preempted);

    let effect = leases.sync(&port, OPEN);

    assert_eq!(effect, Some(LeaseEffect::CloseWorldMap));
    assert_eq!(
        port.take_calls(),
        [
            PortCall::Acquire(Capability::Overlay),
            PortCall::Acquire(Capability::Input),
            PortCall::Holder(Capability::Input),
            PortCall::Release(Capability::Overlay),
        ]
    );
}

#[test]
fn a_map_refused_once_is_asked_again_on_the_next_open() {
    let mut port = FakePort::new();
    let mut leases = started(&port);
    port.outcomes.insert(Capability::Overlay, AcquireOutcome::Denied { holder: ModIndex(3) });
    leases.sync(&port, OPEN);
    leases.sync(&port, CLOSED);
    port.outcomes.clear();
    port.take_calls();

    let effect = leases.sync(&port, OPEN);

    assert_eq!(effect, None);
    assert_eq!(port.take_calls()[0], PortCall::Acquire(Capability::Overlay));
}

#[test]
fn revoked_input_while_open_closes_the_map_and_cursor() {
    let port = FakePort::new();
    let mut leases = started(&port);
    leases.sync(&port, OPEN);
    port.take_calls();

    let effect = leases.on_event(ModEvent::Revoked { capability: Capability::Input, by: ModIndex(2) });
    leases.sync(&port, CLOSED);

    assert_eq!(effect, Some(LeaseEffect::CloseWorldMap));
    assert_eq!(port.take_calls(), [PortCall::Cursor(false), PortCall::Release(Capability::Overlay)]);
}

#[test]
fn suspended_overlay_closes_the_map() {
    let port = FakePort::new();
    let mut leases = started(&port);
    leases.sync(&port, OPEN);

    let effect = leases.on_event(ModEvent::Suspended { capability: Capability::Overlay, by: ModIndex(1) });

    assert_eq!(effect, Some(LeaseEffect::CloseWorldMap));
}

#[test]
fn suspended_hud_has_no_effect_and_makes_no_call() {
    let port = FakePort::new();
    let mut leases = started(&port);

    let effect = leases.on_event(ModEvent::Suspended { capability: Capability::Hud, by: ModIndex(1) });
    leases.sync(&port, CLOSED);

    assert_eq!(effect, None);
    assert_eq!(port.take_calls(), []);
}

#[test]
fn disabled_pauses_and_enabled_resumes_and_reacquires_hud() {
    let port = FakePort::new();
    let mut leases = started(&port);

    assert_eq!(leases.on_event(ModEvent::Disabled), Some(LeaseEffect::Pause));
    leases.sync(&port, CLOSED);
    assert_eq!(port.take_calls(), []);

    assert_eq!(leases.on_event(ModEvent::Enabled), Some(LeaseEffect::Resume));
    leases.sync(&port, CLOSED);
    assert_eq!(port.take_calls(), [PortCall::Acquire(Capability::Hud)]);
}

#[test]
fn unchanged_flags_make_no_port_calls() {
    let port = FakePort::new();
    let mut leases = started(&port);
    leases.sync(&port, OPEN);
    port.take_calls();

    for _ in 0..10 {
        leases.sync(&port, OPEN);
    }

    assert_eq!(port.take_calls(), []);
}

#[test]
fn a_refused_hud_waits_for_capability_available() {
    let mut port = FakePort::new();
    port.outcomes.insert(Capability::Hud, AcquireOutcome::Denied { holder: ModIndex(4) });
    let mut leases = LeaseController::new();
    leases.sync(&port, CLOSED);
    port.take_calls();

    leases.sync(&port, CLOSED);
    assert_eq!(port.take_calls(), []);

    port.outcomes.clear();
    assert_eq!(leases.on_event(ModEvent::CapabilityAvailable(Capability::Hud)), None);
    leases.sync(&port, CLOSED);
    assert_eq!(port.take_calls(), [PortCall::Acquire(Capability::Hud)]);
}

#[test]
fn capability_available_does_not_reopen_the_map() {
    let port = FakePort::new();
    let mut leases = started(&port);

    let effect = leases.on_event(ModEvent::CapabilityAvailable(Capability::Overlay));
    leases.sync(&port, CLOSED);

    assert_eq!(effect, None);
    assert_eq!(port.take_calls(), []);
}

#[test]
fn release_all_turns_the_cursor_off_and_gives_every_lease_back() {
    let port = FakePort::new();
    let mut leases = started(&port);
    leases.sync(&port, OPEN);
    port.take_calls();

    leases.release_all(&port);

    assert_eq!(
        port.take_calls(),
        [
            PortCall::Cursor(false),
            PortCall::Release(Capability::Input),
            PortCall::Release(Capability::Overlay),
            PortCall::Release(Capability::Hud),
        ]
    );
}
