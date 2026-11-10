use minimap_core::services::biome::BiomeIdNames;

#[test]
fn an_unnamed_id_is_asked_for_once_and_its_answer_is_kept() {
    let mut names = BiomeIdNames::new(BiomeIdNames::CAPACITY);
    let mut out = Vec::new();
    names.request(1);
    names.request(1);
    names.take_requests(&mut out);
    assert_eq!(out, vec![1]);
    names.answer(1, "plains");
    assert_eq!(names.name(1), Some("plains"));
    names.request(1);
    names.take_requests(&mut out);
    assert!(out.is_empty());
}

#[test]
fn an_empty_or_overlong_answer_is_no_name_and_is_not_asked_again() {
    let mut names = BiomeIdNames::new(BiomeIdNames::CAPACITY);
    names.answer(9, "");
    names.answer(10, &"x".repeat(65));
    assert_eq!(names.name(9), None);
    assert_eq!(names.name(10), None);
    let mut out = Vec::new();
    names.request(9);
    names.request(10);
    names.take_requests(&mut out);
    assert!(out.is_empty());
}

#[test]
fn requests_leave_at_most_sixteen_per_tick() {
    let mut names = BiomeIdNames::new(BiomeIdNames::CAPACITY);
    for id in 0..40 {
        names.request(id);
    }
    let mut out = Vec::new();
    names.take_requests(&mut out);
    assert_eq!(out.len(), 16);
    names.take_requests(&mut out);
    assert_eq!(out.len(), 16);
    names.take_requests(&mut out);
    assert_eq!(out[..8], [32, 33, 34, 35, 36, 37, 38, 39], "the window wraps to the first waiting IDs");
    assert_eq!(out[8..], [0, 1, 2, 3, 4, 5, 6, 7]);
}

#[test]
fn the_id_map_holds_its_bound() {
    let mut names = BiomeIdNames::new(4);
    for id in 0..10 {
        names.answer(id, "plains");
    }
    assert_eq!(names.len(), 4);
    assert_eq!(names.name(9), None);
    for id in 20..30 {
        names.request(id);
    }
    let mut out = Vec::new();
    names.take_requests(&mut out);
    assert!(out.is_empty(), "a full map queues nothing");
}

#[test]
fn an_id_handed_out_but_not_answered_is_asked_again_until_answered() {
    let mut names = BiomeIdNames::new(BiomeIdNames::CAPACITY);
    let mut out = Vec::new();
    names.request(3);
    names.take_requests(&mut out);
    assert_eq!(out, vec![3]);
    assert!(!names.is_answered(3));
    names.request(3);
    names.take_requests(&mut out);
    assert_eq!(out, vec![3], "a probe may name it next tick");
    names.answer(3, "desert");
    names.take_requests(&mut out);
    assert!(names.is_answered(3) && out.is_empty());
}
