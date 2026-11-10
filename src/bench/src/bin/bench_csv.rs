use std::process::ExitCode;

use minimap_bench::services::Tools;

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    Tools::bench_csv(&args)
}
