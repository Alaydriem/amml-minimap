use minimap_bench::services::Comparison;
use minimap_bench::structs::CsvTable;

#[test]
fn bench_rows_match_on_name_and_full_variant() {
    let before = CsvTable::parse(
        "bench,variant,iterations,mean_ns,p50_ns,p99_ns\nregion_bake,synthetic/scale2_pcf1,320,7000,6000,9000\nregion_bake,other/scale2_pcf1,10,5,5,5\n",
    )
    .unwrap();
    let after = CsvTable::parse(
        "bench,variant,iterations,mean_ns,p50_ns,p99_ns\nregion_bake,synthetic/scale2_pcf1,10,1000,1000,1200\nregion_bake,synthetic/scale4_pcf1,10,3000,3000,3100\n",
    )
    .unwrap();
    let table = Comparison::bench(&before, &after);
    assert!(table.starts_with("| bench | variant | before mean ns | after mean ns | before / after |"), "{table}");
    assert!(table.contains("| region_bake | synthetic/scale2_pcf1 | 7000 | 1000 | 7.00 |"), "{table}");
    assert!(!table.contains("scale4_pcf1"), "rows without a match in the earlier run are left out");
    assert!(!table.contains("| 5 |"), "a different variant prefix is a different row");
}

#[test]
fn telemetry_columns_compare_by_median() {
    let header = "t_ms,render_ms_p50,private_mb";
    let before = CsvTable::parse(&format!("{header}\n1000,2.0,900\n2000,4.0,1100\n")).unwrap();
    let after = CsvTable::parse(&format!("{header}\n1000,0.2,300\n2000,0.4,310\n3000,0.3,305\n")).unwrap();
    let table = Comparison::telemetry(&before, &after);
    assert!(table.starts_with("| column | before median | after median | before / after |"), "{table}");
    assert!(table.contains("| render_ms_p50 | 3.00 | 0.30 | 10.00 |"), "{table}");
    assert!(table.contains("| private_mb | 1000.00 | 305.00 | 3.28 |"), "{table}");
    assert!(!table.contains("| t_ms"));
}

#[test]
fn a_column_only_one_run_has_shows_a_dash_for_the_other() {
    let before = CsvTable::parse("t_ms,render_ms_p50,imgui_vtx\n1000,2.0,640\n").unwrap();
    let after = CsvTable::parse("t_ms,render_ms_p50,ui_vtx\n1000,1.0,900\n").unwrap();

    let table = Comparison::telemetry(&before, &after);

    assert!(table.contains("| render_ms_p50 | 2.00 | 1.00 | 2.00 |"), "{table}");
    assert!(table.contains("| ui_vtx | — | 900.00 | - |"), "{table}");
    assert!(table.contains("| imgui_vtx | 640.00 | — | - |"), "{table}");
}

#[test]
fn short_rows_are_skipped_and_medians_need_numbers() {
    let table = CsvTable::parse("a,b\n1,2\n3\nx,4\n").unwrap();
    assert_eq!(table.rows.len(), 2);
    assert_eq!(table.median("a"), Some(1.0));
    assert_eq!(table.median("b"), Some(3.0));
    assert_eq!(table.median("missing"), None);
}
