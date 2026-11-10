use std::io;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::structs::BenchRow;

// turns Criterion's saved samples (target/criterion/**/new/) into the shared CSV columns
pub struct CriterionCsv;

impl CriterionCsv {
    pub fn collect(criterion_dir: &Path) -> io::Result<Vec<BenchRow>> {
        let mut rows = Vec::new();
        for dir in Self::result_dirs(criterion_dir) {
            if let Some(row) = Self::row(&dir)? {
                rows.push(row);
            }
        }
        rows.sort_by(|a, b| (&a.bench, &a.variant).cmp(&(&b.bench, &b.variant)));
        Ok(rows)
    }

    // one row from a `new/` directory holding benchmark.json and sample.json
    pub fn row(dir: &Path) -> io::Result<Option<BenchRow>> {
        let (Ok(benchmark), Ok(sample)) =
            (std::fs::read_to_string(dir.join("benchmark.json")), std::fs::read_to_string(dir.join("sample.json")))
        else {
            return Ok(None);
        };
        let benchmark: Value = serde_json::from_str(&benchmark).map_err(io::Error::other)?;
        let sample: Value = serde_json::from_str(&sample).map_err(io::Error::other)?;
        let bench = benchmark["group_id"].as_str().unwrap_or("").to_string();
        let mut variant = benchmark["function_id"].as_str().unwrap_or("").to_string();
        if let Some(value) = benchmark["value_str"].as_str() {
            variant = if variant.is_empty() { value.to_string() } else { format!("{variant}/{value}") };
        }
        let numbers = |key: &str| -> Vec<f64> {
            sample[key].as_array().map(|a| a.iter().filter_map(Value::as_f64).collect()).unwrap_or_default()
        };
        let (iters, times) = (numbers("iters"), numbers("times"));
        let total_iters: f64 = iters.iter().sum();
        if total_iters <= 0.0 || iters.len() != times.len() {
            return Ok(None);
        }
        let mut per_iter: Vec<f64> = iters.iter().zip(&times).map(|(i, t)| t / i).collect();
        per_iter.sort_by(f64::total_cmp);
        Ok(Some(BenchRow {
            bench,
            variant,
            iterations: total_iters as u64,
            mean_ns: times.iter().sum::<f64>() / total_iters,
            p50_ns: Self::nearest_rank(&per_iter, 0.5),
            p99_ns: Self::nearest_rank(&per_iter, 0.99),
        }))
    }

    pub fn nearest_rank(sorted: &[f64], q: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let rank = ((q * sorted.len() as f64).ceil() as usize).clamp(1, sorted.len()) - 1;
        sorted[rank]
    }

    fn result_dirs(root: &Path) -> Vec<PathBuf> {
        let mut out = Vec::new();
        let mut stack = vec![root.to_path_buf()];
        while let Some(dir) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&dir) else { continue };
            for entry in entries.flatten() {
                let path = entry.path();
                if !path.is_dir() {
                    continue;
                }
                if path.file_name().is_some_and(|n| n == "new") {
                    out.push(path);
                } else if path.file_name().is_none_or(|n| n != "report") {
                    stack.push(path);
                }
            }
        }
        out
    }
}
