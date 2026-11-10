use std::collections::HashSet;

use crate::structs::LightDirection;
use crate::structs::chunk::HeightChange;
use crate::structs::pos::ChunkPos;
use crate::traits::HeightRanges;

// finds the loaded chunks whose shadow may change after heights inside a rectangle change
pub struct AffectedChunkTracer {
    visited: HashSet<ChunkPos>,
}

impl AffectedChunkTracer {
    pub const MAX_DISTANCE: f32 = 112.0;
    const EPSILON: f32 = 1e-4;

    pub fn new() -> Self {
        Self { visited: HashSet::with_capacity(64) }
    }

    pub fn trace(
        &mut self,
        change: &HeightChange,
        light: LightDirection,
        ranges: &impl HeightRanges,
        out: &mut Vec<ChunkPos>,
    ) {
        self.visited.clear();
        out.clear();
        let rect = change.rect;
        let min_chunk = ChunkPos::from_block(rect.x0, rect.z0, change.dim);
        let max_chunk = ChunkPos::from_block(rect.x1, rect.z1, change.dim);
        for cz in min_chunk.z..=max_chunk.z {
            for cx in min_chunk.x..=max_chunk.x {
                let chunk = ChunkPos::new(cx, cz, change.dim);
                out.push(chunk);
                self.visited.insert(chunk);
            }
        }

        let dir = light.downstream();
        let corners = [(rect.x0, rect.z0), (rect.x1, rect.z0), (rect.x0, rect.z1), (rect.x1, rect.z1)];
        let mut upstream = 0;
        let mut min_dot = f32::MAX;
        for (i, (x, z)) in corners.iter().enumerate() {
            let dot = *x as f32 * dir.0 + *z as f32 * dir.1;
            if dot < min_dot {
                min_dot = dot;
                upstream = i;
            }
        }

        let mut traced: [(i32, i32); 3] = [(0, 0); 3];
        let mut traced_count = 0;
        for (i, corner) in corners.iter().enumerate() {
            if i == upstream {
                continue;
            }
            let origin = (corner.0.clamp(rect.x0, rect.x1), corner.1.clamp(rect.z0, rect.z1));
            if traced[..traced_count].contains(&origin) {
                continue;
            }
            traced[traced_count] = origin;
            traced_count += 1;
            self.trace_ray(origin, dir, change, light.tan_zenith(), ranges, out);
        }
    }

    fn trace_ray(
        &mut self,
        origin: (i32, i32),
        dir: (f32, f32),
        change: &HeightChange,
        tan_zenith: f32,
        ranges: &impl HeightRanges,
        out: &mut Vec<ChunkPos>,
    ) {
        let (ox, oz) = origin;
        let mut cur_x = ox as f32;
        let mut cur_z = oz as f32;
        let mut chunk_x = Self::chunk_of(cur_x, dir.0);
        let mut chunk_z = Self::chunk_of(cur_z, dir.1);
        let mut dist = 0.0f32;

        while dist < Self::MAX_DISTANCE {
            let chunk = ChunkPos::new(chunk_x, chunk_z, change.dim);
            if self.visited.insert(chunk) {
                let (min_x, max_x) = (chunk_x * 16, chunk_x * 16 + 16);
                let (min_z, max_z) = (chunk_z * 16, chunk_z * 16 + 16);
                let near = Self::distance(ox.clamp(min_x, max_x) - ox, oz.clamp(min_z, max_z) - oz);
                let far_x = if (ox - min_x).abs() > (ox - max_x).abs() { min_x } else { max_x };
                let far_z = if (oz - min_z).abs() > (oz - max_z).abs() { min_z } else { max_z };
                let far = Self::distance(far_x - ox, far_z - oz);
                if let Some((chunk_min, chunk_max)) = ranges.height_range(chunk) {
                    if (change.higher as f32) - near * tan_zenith < f32::from(chunk_min) {
                        break;
                    }
                    let always_shadowed = (change.lower as f32) - far * tan_zenith >= f32::from(chunk_max);
                    if !always_shadowed {
                        out.push(chunk);
                    }
                }
            }

            let next_x = if dir.0 >= 0.0 { (chunk_x + 1) * 16 } else { chunk_x * 16 };
            let next_z = if dir.1 >= 0.0 { (chunk_z + 1) * 16 } else { chunk_z * 16 };
            let mut tx = if dir.0 != 0.0 { (next_x as f32 - cur_x) / dir.0 } else { f32::MAX };
            let mut tz = if dir.1 != 0.0 { (next_z as f32 - cur_z) / dir.1 } else { f32::MAX };
            if tx <= 1e-5 {
                tx = f32::MAX;
            }
            if tz <= 1e-5 {
                tz = f32::MAX;
            }
            let step = tx.min(tz);
            if dist + step > Self::MAX_DISTANCE {
                break;
            }
            cur_x += dir.0 * step;
            cur_z += dir.1 * step;
            dist += step;
            chunk_x = Self::chunk_of(cur_x, dir.0);
            chunk_z = Self::chunk_of(cur_z, dir.1);
        }
    }

    // a point on a chunk edge belongs to the chunk the ray is entering
    fn chunk_of(coordinate: f32, direction: f32) -> i32 {
        let nudge = if direction >= 0.0 { Self::EPSILON } else { -Self::EPSILON };
        ((coordinate + nudge) / 16.0).floor() as i32
    }

    fn distance(dx: i32, dz: i32) -> f32 {
        f64::from(dx * dx + dz * dz).sqrt() as f32
    }
}

impl Default for AffectedChunkTracer {
    fn default() -> Self {
        Self::new()
    }
}
