use minimap_bench::services::CriterionCsv;
use minimap_bench::structs::BenchRow;

fn write_result(
    root: &std::path::Path,
    group: &str,
    function: &str,
    value: Option<&str>,
    iters: &[f64],
    times: &[f64],
) {
    let dir = root.join(group).join(function).join(value.unwrap_or("")).join("new");
    std::fs::create_dir_all(&dir).unwrap();
    let benchmark = serde_json::json!({ "group_id": group, "function_id": function, "value_str": value });
    std::fs::write(dir.join("benchmark.json"), benchmark.to_string()).unwrap();
    let sample = serde_json::json!({ "sampling_mode": "Linear", "iters": iters, "times": times });
    std::fs::write(dir.join("sample.json"), sample.to_string()).unwrap();
}

#[test]
fn saved_samples_become_rows_with_mean_and_quantiles() {
    let dir = tempfile::tempdir().unwrap();
    write_result(
        dir.path(),
        "chunk_bake",
        "synthetic",
        Some("scale2_pcf1"),
        &[1.0, 2.0, 3.0, 4.0],
        &[100.0, 400.0, 900.0, 1600.0],
    );
    write_result(dir.path(), "draw_list", "minimap", None, &[10.0], &[5000.0]);
    std::fs::create_dir_all(dir.path().join("report")).unwrap();
    let rows = CriterionCsv::collect(dir.path()).unwrap();
    assert_eq!(rows.len(), 2);
    let bake = &rows[0];
    assert_eq!((bake.bench.as_str(), bake.variant.as_str()), ("chunk_bake", "synthetic/scale2_pcf1"));
    assert_eq!(bake.iterations, 10);
    // per-iteration times are 100, 200, 300, 400 ns
    assert_eq!(bake.mean_ns, 300.0);
    assert_eq!(bake.p50_ns, 200.0);
    assert_eq!(bake.p99_ns, 400.0);
    assert_eq!(rows[1].variant, "minimap");
    assert_eq!(rows[1].to_csv(), "draw_list,minimap,10,500,500,500");
}

#[test]
fn the_csv_columns_are_the_shared_ones() {
    assert_eq!(BenchRow::HEADER, "bench,variant,iterations,mean_ns,p50_ns,p99_ns");
}

#[test]
fn a_directory_without_samples_gives_no_rows() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join("x").join("new")).unwrap();
    assert!(CriterionCsv::collect(dir.path()).unwrap().is_empty());
}
