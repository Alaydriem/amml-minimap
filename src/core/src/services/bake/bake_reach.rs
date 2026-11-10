use crate::services::bake::ShadowCaster;
use crate::structs::LightDirection;

// the chunk offsets a bake can read: shadow rays toward the light, plus one chunk all round for PCF and bevel
pub struct BakeReach;

impl BakeReach {
    // inclusive (dx_min, dx_max, dz_min, dz_max) around a chunk
    pub fn offsets(light: LightDirection) -> (i32, i32, i32, i32) {
        let (sun_x, _, sun_z) = light.sun();
        let rays = (ShadowCaster::MAX_STEPS + 15) / 16 + 1;
        let low = |v: f32| if v < 0.0 { -rays } else { -1 };
        let high = |v: f32| if v > 0.0 { rays } else { 1 };
        (low(sun_x), high(sun_x), low(sun_z), high(sun_z))
    }
}
