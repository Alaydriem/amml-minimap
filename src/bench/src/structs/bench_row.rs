// one line of the bench CSV that bench-report writes and report compares
#[derive(Debug, Clone, PartialEq)]
pub struct BenchRow {
    pub bench: String,
    pub variant: String,
    pub iterations: u64,
    pub mean_ns: f64,
    pub p50_ns: f64,
    pub p99_ns: f64,
}

impl BenchRow {
    pub const HEADER: &'static str = "bench,variant,iterations,mean_ns,p50_ns,p99_ns";

    pub fn to_csv(&self) -> String {
        format!(
            "{},{},{},{:.0},{:.0},{:.0}",
            self.bench, self.variant, self.iterations, self.mean_ns, self.p50_ns, self.p99_ns
        )
    }
}
