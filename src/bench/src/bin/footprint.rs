use minimap_bench::services::FootprintReport;

fn main() {
    for (name, value) in FootprintReport::rows() {
        println!("{name},{value}");
    }
}
