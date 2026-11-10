use minimap_core::errors::WaypointError;
use minimap_core::services::waypoints::WaypointStore;
use minimap_core::structs::waypoint::{WaypointEdit, WaypointIcon, WaypointSource};

const NOW: u64 = 1_792_138_360;

fn store(capacity: usize) -> WaypointStore {
    WaypointStore::new(capacity, 7)
}

#[test]
fn a_blank_name_becomes_the_first_free_waypoint_number() {
    let mut store = store(8);
    store.add("  ", 0, (1, 64, 1), WaypointSource::Hotkey, NOW).unwrap();
    store.add("", 0, (2, 64, 2), WaypointSource::Hotkey, NOW).unwrap();
    store.add("Waypoint 4", 0, (3, 64, 3), WaypointSource::Manual, NOW).unwrap();
    store.add("", 0, (4, 64, 4), WaypointSource::Hotkey, NOW).unwrap();
    let names: Vec<&str> = store.waypoints().iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, vec!["Waypoint 1", "Waypoint 2", "Waypoint 4", "Waypoint 3"]);
}

#[test]
fn names_are_trimmed_and_cut_to_thirty_two_characters() {
    let mut store = store(8);
    let long = format!("  {}  ", "é".repeat(40));
    let id = store.add(&long, 0, (0, 0, 0), WaypointSource::Manual, NOW).unwrap();
    let name = &store.get(id).unwrap().name;
    assert_eq!(name.chars().count(), WaypointStore::MAX_NAME_CHARS);
    assert!(name.chars().all(|c| c == 'é'));
}

#[test]
fn colours_follow_the_palette_and_wrap_after_the_last() {
    let mut store = store(16);
    let count = WaypointStore::PALETTE.len();
    for i in 0..=count {
        store.add("", 0, (i as i32, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    }
    let colours: Vec<[u8; 4]> = store.waypoints().iter().map(|w| w.colour).collect();
    assert_eq!(&colours[..count], &WaypointStore::PALETTE);
    assert_eq!(colours[count], WaypointStore::PALETTE[0]);
}

#[test]
fn adding_past_the_cap_is_refused_and_keeps_the_oldest() {
    let mut store = store(3);
    for i in 0..3 {
        store.add("", 0, (i, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    }
    assert_eq!(store.add("late", 0, (9, 64, 9), WaypointSource::Manual, NOW), Err(WaypointError::Full(3)));
    assert_eq!(store.len(), 3);
    assert_eq!(store.waypoints()[0].x, 0);
}

#[test]
fn a_new_death_moves_the_one_death_point() {
    let mut store = store(8);
    let first = store.set_death(0, (10, 70, -5), NOW);
    let second = store.set_death(1, (-3, 40, 8), NOW + 60);
    assert_eq!(first, second);
    assert_eq!(store.len(), 1);
    let death = store.death_point().unwrap();
    assert_eq!((death.dim, death.x, death.y, death.z, death.created), (1, -3, 40, 8, NOW + 60));
    assert_eq!(death.icon, WaypointIcon::Skull);
    assert_eq!(death.name, WaypointStore::DEATH_NAME);
}

#[test]
fn a_death_is_kept_even_when_the_list_is_full() {
    let mut store = store(2);
    store.add("", 0, (0, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    store.add("", 0, (1, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    store.set_death(0, (5, 64, 5), NOW);
    assert_eq!(store.len(), 3);
    assert!(store.add("", 0, (2, 64, 0), WaypointSource::Hotkey, NOW).is_err());
    store.set_death(0, (6, 64, 6), NOW);
    assert_eq!(store.len(), 3);
}

#[test]
fn ids_are_unique_and_never_zero() {
    let mut store = store(512);
    for i in 0..512 {
        store.add("", 0, (i, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    }
    let mut ids: Vec<u64> = store.waypoints().iter().map(|w| w.id.0).collect();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), 512);
    assert!(ids[0] != 0);
}

#[test]
fn loaded_waypoints_join_new_ones_and_drop_duplicates_extra_deaths_and_overflow() {
    let mut source = store(8);
    let kept = source.add("Home", 0, (1, 64, 1), WaypointSource::Manual, NOW).unwrap();
    source.add("Mine", -1, (2, 30, 2), WaypointSource::Manual, NOW).unwrap();
    source.set_death(0, (3, 64, 3), NOW);
    let mut file = source.waypoints().to_vec();
    file.push(file[0].clone());

    let mut target = WaypointStore::new(3, 99);
    target.set_death(0, (9, 64, 9), NOW);
    target.add("Fresh", 0, (8, 64, 8), WaypointSource::Hotkey, NOW).unwrap();
    let dropped = target.merge_loaded(file);
    // Mine has no room, the file's death loses to the new one, and Home's copy is a duplicate
    assert_eq!(dropped, 3);
    let names: Vec<&str> = target.waypoints().iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, vec![WaypointStore::DEATH_NAME, "Fresh", "Home"]);
    assert!(target.get(kept).is_some());
}

#[test]
fn loaded_names_are_cleaned_like_new_ones() {
    let mut source = store(4);
    source.add("x", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    let mut file = source.waypoints().to_vec();
    file[0].name = "   ".to_string();
    let mut target = store(4);
    target.merge_loaded(file);
    assert_eq!(target.waypoints()[0].name, "Waypoint 1");
}

#[test]
fn changes_mark_the_store_dirty_once_but_loading_and_clearing_do_not() {
    let mut store = store(4);
    let start = store.revision();
    store.merge_loaded(Vec::new());
    assert!(!store.take_dirty());
    store.add("", 0, (0, 64, 0), WaypointSource::Hotkey, NOW).unwrap();
    assert!(store.take_dirty());
    assert!(!store.take_dirty());
    store.set_death(0, (1, 64, 1), NOW);
    store.clear();
    assert!(!store.take_dirty());
    assert!(store.is_empty());
    assert!(store.revision() > start + 3);
}

#[test]
fn a_removed_waypoint_is_restored_at_its_old_place() {
    let mut store = store(8);
    for name in ["A", "B", "C"] {
        store.add(name, 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    }
    let b = store.waypoints()[1].id;
    let (index, removed) = store.remove(b).unwrap();
    assert_eq!((index, store.len()), (1, 2));
    assert!(store.remove(b).is_none());
    assert!(store.restore(index, removed.clone()));
    let names: Vec<&str> = store.waypoints().iter().map(|w| w.name.as_str()).collect();
    assert_eq!(names, vec!["A", "B", "C"]);
    assert!(!store.restore(index, removed));
}

#[test]
fn a_restore_is_refused_when_the_list_filled_up_meanwhile() {
    let mut store = store(2);
    let a = store.add("A", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    store.add("B", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    let (index, removed) = store.remove(a).unwrap();
    store.add("C", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    assert!(!store.restore(index, removed));
    assert_eq!(store.len(), 2);
}

#[test]
fn an_update_cleans_the_name_and_a_blank_name_keeps_the_old_one() {
    let mut store = store(4);
    let id = store.add("Home", 0, (0, 64, 0), WaypointSource::Manual, NOW).unwrap();
    store.take_dirty();
    let edit = |name: &str| WaypointEdit {
        name: name.to_string(),
        colour: [1, 2, 3, 4],
        show_on_edge: false,
        show_distance: false,
    };
    assert!(store.update(id, &edit("   ")));
    assert_eq!(store.get(id).unwrap().name, "Home");
    assert!(store.take_dirty());
    store.update(id, &edit(&format!(" {} ", "x".repeat(40))));
    let waypoint = store.get(id).unwrap();
    assert_eq!(waypoint.name, "x".repeat(32));
    assert_eq!(waypoint.colour, [1, 2, 3, 4]);
    assert!(!waypoint.show_on_edge && !waypoint.show_distance);
    assert!(!store.update(minimap_core::structs::waypoint::WaypointId(1), &edit("Gone")));
}
