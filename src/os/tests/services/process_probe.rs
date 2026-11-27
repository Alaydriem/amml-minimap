use minimap_os::services::ProcessProbe;

#[test]
fn this_process_counts_as_running() {
    assert!(ProcessProbe::is_running(std::process::id()));
}

#[test]
fn an_ended_process_does_not_count_as_running() {
    let mut child = std::process::Command::new("cmd").args(["/C", "exit 0"]).spawn().unwrap();
    let pid = child.id();
    child.wait().unwrap();
    drop(child);

    assert!(!ProcessProbe::is_running(pid));
}
