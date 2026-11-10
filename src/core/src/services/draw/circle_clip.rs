// clips a line segment to the inside of a circle
pub struct CircleClip;

impl CircleClip {
    pub fn clip(a: (f32, f32), b: (f32, f32), centre: (f32, f32), radius: f32) -> Option<((f32, f32), (f32, f32))> {
        let inside = |p: (f32, f32)| {
            let (dx, dy) = (p.0 - centre.0, p.1 - centre.1);
            dx * dx + dy * dy <= radius * radius
        };
        let d = (b.0 - a.0, b.1 - a.1);
        let f = (a.0 - centre.0, a.1 - centre.1);
        let qa = d.0 * d.0 + d.1 * d.1;
        if qa < 0.0001 {
            return inside(a).then_some((a, b));
        }
        let qb = 2.0 * (f.0 * d.0 + f.1 * d.1);
        let qc = f.0 * f.0 + f.1 * f.1 - radius * radius;
        let disc = qb * qb - 4.0 * qa * qc;
        if disc < 0.0 {
            return None;
        }
        let root = disc.sqrt();
        let t1 = ((-qb - root) / (2.0 * qa)).clamp(0.0, 1.0);
        let t2 = ((-qb + root) / (2.0 * qa)).clamp(0.0, 1.0);
        if t1 >= t2 {
            return None;
        }
        Some(((a.0 + d.0 * t1, a.1 + d.1 * t1), (a.0 + d.0 * t2, a.1 + d.1 * t2)))
    }
}
