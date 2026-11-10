use minimap_core::services::draw::MapRotation;

fn close(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

#[test]
fn a_quarter_turn_moves_east_of_the_pivot_to_south_of_it() {
    let turn = MapRotation::new(std::f32::consts::FRAC_PI_2, (100.0, 50.0));
    assert!(close(turn.apply((110.0, 50.0)), (100.0, 60.0)));
    assert!(close(turn.apply((100.0, 50.0)), (100.0, 50.0)));
    assert!(turn.is_turned());
    assert!(!MapRotation::new(0.0, (0.0, 0.0)).is_turned());
}

#[test]
fn inverting_a_vector_undoes_the_turn() {
    let turn = MapRotation::new(0.7, (0.0, 0.0));
    let moved = turn.apply((3.0, -4.0));
    assert!(close(turn.invert_vector(moved), (3.0, -4.0)));
}

#[test]
fn player_up_turns_the_player_facing_to_the_top_of_the_screen() {
    for yaw in [0.0f32, 37.0, 180.0, -95.0] {
        let render = (yaw + 180.0).to_radians();
        let facing = MapRotation::new(render, (0.0, 0.0)).apply((0.0, -1.0));
        let up = MapRotation::new(MapRotation::player_up(render), (0.0, 0.0)).apply(facing);
        assert!(close(up, (0.0, -1.0)), "yaw {yaw}: {up:?}");
    }
}
