use crate::config::SmoothCameraConfig;
use crate::structs::view::{CameraPose, PlayerSnapshot};

// spring-damper follow of the player position and yaw
pub struct SmoothCamera {
    pose: CameraPose,
    velocity: (f32, f32, f32),
    spring: f32,
    damping: f32,
    max_dt: f32,
    following: Option<i32>,
}

impl SmoothCamera {
    pub fn new(config: &SmoothCameraConfig) -> Self {
        Self {
            pose: CameraPose::default(),
            velocity: (0.0, 0.0, 0.0),
            spring: config.spring_strength,
            damping: config.damping,
            max_dt: config.max_delta_time,
            following: None,
        }
    }

    pub fn pose(&self) -> CameraPose {
        self.pose
    }

    // the next update jumps to the player, as on world entry
    pub fn reset(&mut self) {
        self.following = None;
    }

    // jumps to the player on the first update and after a dimension change
    pub fn update(&mut self, player: &PlayerSnapshot, dt: f32) -> CameraPose {
        if !player.present {
            self.following = None;
            return self.pose;
        }
        if self.following != Some(player.dim) {
            self.pose = CameraPose { x: player.x, z: player.z, yaw: player.yaw };
            self.velocity = (0.0, 0.0, 0.0);
            self.following = Some(player.dim);
            return self.pose;
        }
        let dt = dt.clamp(0.0, self.max_dt);
        let mut yaw_diff = player.yaw - self.pose.yaw;
        while yaw_diff > 180.0 {
            yaw_diff -= 360.0;
        }
        while yaw_diff < -180.0 {
            yaw_diff += 360.0;
        }
        let target_yaw = self.pose.yaw + yaw_diff;
        let (vx, vz, vyaw) = self.velocity;
        let (x, vx) = self.spring_step(player.x, self.pose.x, vx, dt);
        let (z, vz) = self.spring_step(player.z, self.pose.z, vz, dt);
        let (yaw, vyaw) = self.spring_step(target_yaw, self.pose.yaw, vyaw, dt);
        self.pose = CameraPose { x, z, yaw };
        self.velocity = (vx, vz, vyaw);
        self.pose
    }

    fn spring_step(&self, target: f32, current: f32, velocity: f32, dt: f32) -> (f32, f32) {
        let acceleration = (target - current) * self.spring - velocity * self.damping;
        let velocity = velocity + acceleration * dt;
        (current + velocity * dt, velocity)
    }
}
