use minimap_core::config::SmoothCameraConfig;
use minimap_core::services::camera::SmoothCamera;
use minimap_core::structs::view::PlayerSnapshot;

fn player(x: f32, yaw: f32, dim: i32) -> PlayerSnapshot {
    PlayerSnapshot { present: true, x, y: 64.0, z: 0.0, yaw, dim }
}

#[test]
fn the_first_update_jumps_to_the_player() {
    let mut camera = SmoothCamera::new(&SmoothCameraConfig::default());
    let pose = camera.update(&player(100.0, 30.0, 0), 0.016);
    assert_eq!((pose.x, pose.yaw), (100.0, 30.0));
}

#[test]
fn the_camera_approaches_a_moved_player_without_jumping() {
    let mut camera = SmoothCamera::new(&SmoothCameraConfig::default());
    camera.update(&player(0.0, 0.0, 0), 0.016);
    let pose = camera.update(&player(10.0, 0.0, 0), 0.016);
    assert!(pose.x > 0.0 && pose.x < 1.0, "{}", pose.x);
    for _ in 0..600 {
        camera.update(&player(10.0, 0.0, 0), 0.016);
    }
    assert!((camera.pose().x - 10.0).abs() < 0.01);
}

#[test]
fn yaw_takes_the_short_way_round() {
    let mut camera = SmoothCamera::new(&SmoothCameraConfig::default());
    camera.update(&player(0.0, 170.0, 0), 0.016);
    for _ in 0..600 {
        camera.update(&player(0.0, -170.0, 0), 0.016);
    }
    assert!((camera.pose().yaw - 190.0).abs() < 0.1, "{}", camera.pose().yaw);
}

#[test]
fn a_long_frame_is_clamped_and_a_dimension_change_jumps() {
    let mut camera = SmoothCamera::new(&SmoothCameraConfig::default());
    camera.update(&player(0.0, 0.0, 0), 0.016);
    let pose = camera.update(&player(10.0, 0.0, 0), 5.0);
    assert!(pose.x < 10.0);
    let jumped = camera.update(&player(500.0, 0.0, 1), 0.016);
    assert_eq!(jumped.x, 500.0);
}
