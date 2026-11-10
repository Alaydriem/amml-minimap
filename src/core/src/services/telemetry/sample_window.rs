// the samples of one telemetry interval; bounded, so a stalled writer cannot grow it
pub struct SampleWindow {
    values: Vec<f32>,
    scratch: Vec<f32>,
    capacity: usize,
}

impl SampleWindow {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self { values: Vec::with_capacity(capacity), scratch: Vec::with_capacity(capacity), capacity }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    // a full window keeps its first samples and ignores the rest
    pub fn push(&mut self, value: f32) {
        if self.values.len() < self.capacity {
            self.values.push(value);
        }
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    pub fn mean(&self) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.values.iter().sum::<f32>() / self.values.len() as f32
    }

    // nearest-rank quantile, q in 0..=1; 0 when empty
    pub fn quantile(&mut self, q: f32) -> f32 {
        if self.values.is_empty() {
            return 0.0;
        }
        self.scratch.clear();
        self.scratch.extend_from_slice(&self.values);
        let n = self.scratch.len();
        let rank = ((q.clamp(0.0, 1.0) * n as f32).ceil() as usize).clamp(1, n) - 1;
        let (_, value, _) = self.scratch.select_nth_unstable_by(rank, f32::total_cmp);
        *value
    }
}
