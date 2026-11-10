use minimap_core::services::telemetry::SampleWindow;

#[test]
fn quantiles_use_the_nearest_rank() {
    let mut window = SampleWindow::new(1000);
    for v in (1..=100).rev() {
        window.push(v as f32);
    }
    assert_eq!(window.quantile(0.5), 50.0);
    assert_eq!(window.quantile(0.99), 99.0);
    assert_eq!(window.quantile(1.0), 100.0);
    assert_eq!(window.quantile(0.0), 1.0);
    assert!((window.mean() - 50.5).abs() < 1e-4);
}

#[test]
fn an_empty_window_reports_zero() {
    let mut window = SampleWindow::new(8);
    assert_eq!(window.quantile(0.5), 0.0);
    assert_eq!(window.mean(), 0.0);
}

#[test]
fn a_full_window_ignores_further_samples_until_cleared() {
    let mut window = SampleWindow::new(4);
    for v in [1.0, 2.0, 3.0, 4.0, 1000.0, 1000.0] {
        window.push(v);
    }
    assert_eq!(window.len(), 4);
    assert_eq!(window.quantile(1.0), 4.0);
    window.clear();
    assert!(window.is_empty());
    window.push(7.0);
    assert_eq!(window.quantile(0.5), 7.0);
}
