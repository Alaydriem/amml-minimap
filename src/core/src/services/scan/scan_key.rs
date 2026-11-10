// queue order: due tick, then spiral rank, then position
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct ScanKey {
    pub(crate) due: u64,
    pub(crate) rank: u32,
    pub(crate) x: i32,
    pub(crate) z: i32,
}

impl ScanKey {
    pub(crate) fn new(due: u64, rank: u32, x: i32, z: i32) -> Self {
        Self { due, rank, x, z }
    }

    // Chebyshev distance, then Manhattan distance, then dz, then dx
    pub(crate) fn spiral_rank(dx: i32, dz: i32, radius: i32) -> u32 {
        let chebyshev = dx.abs().max(dz.abs()) as u32;
        let manhattan = (dx.abs() + dz.abs()) as u32;
        let z = (dz + radius) as u32;
        let x = (dx + radius) as u32;
        (chebyshev << 24) | (manhattan << 16) | (z << 8) | x
    }
}
