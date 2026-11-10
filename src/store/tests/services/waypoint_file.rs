use minimap_core::structs::waypoint::{Waypoint, WaypointIcon, WaypointId, WaypointSource};
use minimap_store::errors::StoreError;
use minimap_store::services::WaypointFile;

const HOME: &str = r#"{ "version": 1, "waypoints": [ { "id": "7f3c1a9e00000000", "name": "Home", "dim": 0,
  "x": 38, "y": 71, "z": -26, "colour": [80, 170, 255, 255], "icon": "diamond",
  "showOnEdge": true, "showDistance": true, "source": "manual", "created": 1792138360 } ] }"#;

fn home() -> Waypoint {
    Waypoint {
        id: WaypointId(0x7f3c_1a9e_0000_0000),
        name: "Home".to_string(),
        dim: 0,
        x: 38,
        y: 71,
        z: -26,
        colour: [80, 170, 255, 255],
        icon: WaypointIcon::Diamond,
        show_on_edge: true,
        show_distance: true,
        source: WaypointSource::Manual,
        created: 1_792_138_360,
    }
}

#[test]
fn a_version_one_file_reads_every_field() {
    let (waypoints, warnings) = WaypointFile::parse(HOME).unwrap();
    assert_eq!(waypoints, vec![home()]);
    assert!(warnings.is_empty());
}

#[test]
fn ids_are_written_as_sixteen_hex_digits_and_the_version_is_one() {
    let mut low = home();
    low.id = WaypointId(0x2a);
    let json = WaypointFile::to_json(&[low]).unwrap();
    let value: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(value["version"], 1);
    assert_eq!(value["waypoints"][0]["id"], "000000000000002a");
    assert_eq!(value["waypoints"][0]["showOnEdge"], true);
    assert_eq!(value["waypoints"][0]["icon"], "diamond");
}

// one good entry, then a bad id, a colour channel over 255, and an unknown icon
fn with_bad_entries() -> String {
    let entry = |id: &str, colour: &str, icon: &str| {
        format!(
            r#"{{ "id": "{id}", "name": "B", "dim": 0, "x": 0, "y": 0, "z": 0, "colour": {colour}, "icon": "{icon}",
  "showOnEdge": true, "showDistance": true, "source": "manual", "created": 0 }}"#
        )
    };
    let bad = [
        entry("zz", "[1, 2, 3, 4]", "diamond"),
        entry("01", "[300, 0, 0, 255]", "diamond"),
        entry("02", "[1, 2, 3, 4]", "star"),
    ];
    HOME.replace("} ] }", &format!("}}, {} ] }}", bad.join(", ")))
}

#[test]
fn entries_with_a_bad_id_colour_or_icon_are_dropped_with_a_warning_each() {
    let (waypoints, warnings) = WaypointFile::parse(&with_bad_entries()).unwrap();
    assert_eq!(waypoints, vec![home()]);
    assert_eq!(warnings.len(), 3);
    assert!(warnings[0].starts_with("waypoint 1 dropped"));
    assert!(warnings[2].starts_with("waypoint 3 dropped"));
}

#[test]
fn another_version_or_a_missing_list_is_not_a_waypoints_file() {
    assert!(matches!(
        WaypointFile::parse(r#"{ "version": 2, "waypoints": [] }"#),
        Err(StoreError::UnsupportedVersion(2))
    ));
    assert!(matches!(WaypointFile::parse(r#"{ "version": 1 }"#), Err(StoreError::NotWaypointFile)));
    assert!(matches!(WaypointFile::parse("[1, 2"), Err(StoreError::Json(_))));
}

#[test]
fn a_missing_file_loads_as_no_waypoints() {
    let dir = tempfile::tempdir().unwrap();
    assert!(WaypointFile::load(&dir.path().join(WaypointFile::NAME)).unwrap().is_empty());
}

#[test]
fn an_unreadable_file_is_set_aside_and_loads_as_no_waypoints() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join(WaypointFile::NAME);
    std::fs::write(&path, "{ not json").unwrap();
    assert!(WaypointFile::load(&path).unwrap().is_empty());
    assert!(!path.exists());
    assert_eq!(std::fs::read_to_string(dir.path().join("waypoints.json.invalid")).unwrap(), "{ not json");
}

#[test]
fn a_save_replaces_the_file_and_leaves_no_temporary_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("world").join(WaypointFile::NAME);
    WaypointFile::save(&path, &[home(), home()]).unwrap();
    WaypointFile::save(&path, &[home()]).unwrap();
    assert_eq!(WaypointFile::load(&path).unwrap(), vec![home()]);
    assert!(!path.with_extension("json.tmp").exists());
}
