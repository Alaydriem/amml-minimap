use crate::structs::pos::RegionPos;

// the order the world map hands out textures in: the regions nearest the view centre first, so a full budget
// leaves out the far edge of the view, not a random part of it
pub struct RegionOrder;

impl RegionOrder {
    // indices into `regions`, nearest the centre (in blocks) first; `out` keeps its capacity
    pub fn nearest_first(regions: &[RegionPos], centre: (f64, f64), out: &mut Vec<usize>) {
        out.clear();
        out.extend(0..regions.len());
        let distance = |region: &RegionPos| {
            let x = (f64::from(region.x) + 0.5) * f64::from(RegionPos::BLOCKS) - centre.0;
            let z = (f64::from(region.z) + 0.5) * f64::from(RegionPos::BLOCKS) - centre.1;
            x * x + z * z
        };
        out.sort_unstable_by(|a, b| distance(&regions[*a]).total_cmp(&distance(&regions[*b])));
    }
}
