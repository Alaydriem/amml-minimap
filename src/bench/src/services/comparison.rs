use std::fmt::Write;

use crate::structs::CsvTable;

// side-by-side tables of an earlier run's CSV file and a later one's, as Markdown
pub struct Comparison;

impl Comparison {
    // bench rows match on the bench name and the full variant
    pub fn bench(before: &CsvTable, after: &CsvTable) -> String {
        let mut out = String::from(
            "| bench | variant | before mean ns | after mean ns | before / after |\n|---|---|---|---|---|\n",
        );
        let (Some(ab), Some(av), Some(am)) = (after.column("bench"), after.column("variant"), after.column("mean_ns"))
        else {
            return out;
        };
        let (Some(bb), Some(bv), Some(bm)) =
            (before.column("bench"), before.column("variant"), before.column("mean_ns"))
        else {
            return out;
        };
        for row in &after.rows {
            let matching = before.rows.iter().find(|b| b[bb] == row[ab] && b[bv] == row[av]);
            let (Ok(later), Some(earlier)) = (row[am].parse::<f64>(), matching.and_then(|b| b[bm].parse::<f64>().ok()))
            else {
                continue;
            };
            let ratio = if later > 0.0 { earlier / later } else { 0.0 };
            let _ = writeln!(out, "| {} | {} | {earlier:.0} | {later:.0} | {ratio:.2} |", row[ab], row[av]);
        }
        out
    }

    // the median of every numeric telemetry column except time, for each run; a column only one
    // run has shows a dash for the other
    pub fn telemetry(before: &CsvTable, after: &CsvTable) -> String {
        let mut out = String::from("| column | before median | after median | before / after |\n|---|---|---|---|\n");
        let only_before = before.header.iter().filter(|h| after.column(h).is_none());
        for name in after.header.iter().chain(only_before).filter(|h| h.as_str() != "t_ms") {
            let (earlier, later) = (before.median(name), after.median(name));
            if earlier.is_none() && later.is_none() {
                continue;
            }
            let ratio = match (earlier, later) {
                (Some(earlier), Some(later)) if later > 0.0 => format!("{:.2}", earlier / later),
                _ => "-".to_string(),
            };
            let cell = |value: Option<f64>| value.map_or_else(|| "—".to_string(), |v| format!("{v:.2}"));
            let _ = writeln!(out, "| {name} | {} | {} | {ratio} |", cell(earlier), cell(later));
        }
        out
    }
}
