use std::path::Path;
use std::process::ExitCode;

use crate::services::{BenchInputs, Comparison, CriterionCsv, SyntheticCapture};
use crate::structs::{BenchRow, CsvTable};

// the command-line tools of minimap-bench; stdout is their output
pub struct Tools;

impl Tools {
    pub const CAPTURE_PROCEDURE: &'static str = "Recording a probe capture (nothing runs from this task):
  1. In the mod manager's panel, on the Minimap page, turn on Debug > Capture probes. Or close the game
     and set debug { capture_probes = true } in mods/amml/@alaydriem/minimap/config/config.hcl.
  2. Start the game and load the world to record. Play the route: walk and fly over water and
     hills, open the world map (M), pan and zoom it, close it. For a nether capture, go through a portal.
  3. Leave the world and close the game, so the capture file is closed cleanly.
  4. The capture is in the mod data folder under captures/capture-<time>.lmcap. Copy it to fixtures/
     with a name for the route, for example fixtures/overworld-hills.lmcap.
  5. Turn Capture probes off again. Run mise run e2e and mise run bench: both tasks
     pick up every capture in fixtures/.";

    // bench-csv <criterion dir> <out.csv>
    pub fn bench_csv(args: &[String]) -> ExitCode {
        let [criterion, out] = args else { return Self::usage("bench-csv <criterion dir> <out.csv>") };
        let rows = match CriterionCsv::collect(Path::new(criterion)) {
            Ok(rows) => rows,
            Err(error) => return Self::fail(&format!("cannot read {criterion}: {error}")),
        };
        let mut text = String::from(BenchRow::HEADER);
        for row in &rows {
            text.push('\n');
            text.push_str(&row.to_csv());
        }
        text.push('\n');
        if let Err(error) = Self::write(Path::new(out), &text) {
            return Self::fail(&format!("cannot write {out}: {error}"));
        }
        println!("{} benches written to {out}", rows.len());
        ExitCode::SUCCESS
    }

    // capture-tool synth <out.lmcap> | unpack <in.lmcap> <out.lmraw> | e2e <fixtures dir> <out dir> | procedure
    pub fn capture_tool(args: &[String]) -> ExitCode {
        match args {
            [mode, out] if mode == "synth" => match SyntheticCapture::write(Path::new(out)) {
                Ok(report) => {
                    println!("{} events written to {out}", report.written);
                    ExitCode::SUCCESS
                }
                Err(error) => Self::fail(&format!("cannot write {out}: {error}")),
            },
            [mode, input, out] if mode == "unpack" => {
                let unpacked = minimap_runtime::services::CaptureReader::open(Path::new(input))
                    .and_then(|reader| reader.unpack_to(Path::new(out)));
                match unpacked {
                    Ok(count) => {
                        println!("{count} events unpacked to {out}");
                        ExitCode::SUCCESS
                    }
                    Err(error) => Self::fail(&format!("cannot unpack {input}: {error}")),
                }
            }
            [mode, fixtures, out] if mode == "e2e" => Self::prepare_e2e(Path::new(fixtures), Path::new(out)),
            [mode] if mode == "procedure" => {
                println!("{}", Self::CAPTURE_PROCEDURE);
                ExitCode::SUCCESS
            }
            _ => Self::usage(
                "capture-tool synth <out.lmcap> | unpack <in.lmcap> <out.lmraw> | e2e <fixtures dir> <out dir> | procedure",
            ),
        }
    }

    // report bench <before.csv> <after.csv> | telemetry <before.csv> <after.csv>
    pub fn report(args: &[String]) -> ExitCode {
        let [kind, before, after] = args else {
            return Self::usage("report bench <before.csv> <after.csv> | telemetry <before.csv> <after.csv>");
        };
        let read = |path: &str| std::fs::read_to_string(path).ok().and_then(|text| CsvTable::parse(&text));
        let (Some(earlier), Some(later)) = (read(before), read(after)) else {
            return Self::fail(&format!("cannot read {before} and {after} as CSV with a header row"));
        };
        match kind.as_str() {
            "bench" => println!("{}", Comparison::bench(&earlier, &later)),
            "telemetry" => println!("{}", Comparison::telemetry(&earlier, &later)),
            _ => return Self::usage("report bench <before.csv> <after.csv> | telemetry <before.csv> <after.csv>"),
        }
        ExitCode::SUCCESS
    }

    pub(crate) fn usage(text: &str) -> ExitCode {
        eprintln!("usage: {text}");
        ExitCode::from(2)
    }

    pub(crate) fn fail(text: &str) -> ExitCode {
        eprintln!("{text}");
        ExitCode::FAILURE
    }

    // the synthetic walk and every fixture capture, unpacked for the C++ e2e harness
    fn prepare_e2e(fixtures: &Path, out: &Path) -> ExitCode {
        if let Err(error) = std::fs::create_dir_all(out) {
            return Self::fail(&format!("cannot create {}: {error}", out.display()));
        }
        let synthetic = out.join("synthetic-walk.lmcap");
        if let Err(error) = SyntheticCapture::write(&synthetic) {
            return Self::fail(&format!("cannot write {}: {error}", synthetic.display()));
        }
        let mut captures = vec![synthetic];
        captures.extend(BenchInputs::captures(fixtures));
        for capture in captures {
            let raw = out.join(capture.with_extension("lmraw").file_name().unwrap_or_default());
            let unpacked = minimap_runtime::services::CaptureReader::open(&capture).and_then(|r| r.unpack_to(&raw));
            match unpacked {
                Ok(count) => println!("{count} events unpacked to {}", raw.display()),
                Err(error) => return Self::fail(&format!("cannot unpack {}: {error}", capture.display())),
            }
        }
        ExitCode::SUCCESS
    }

    fn write(path: &Path, text: &str) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(path, text)
    }
}
