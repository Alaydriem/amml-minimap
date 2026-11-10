use crate::services::cave::LayerIds;
use crate::structs::export::ExportKind;

// export file names: <world>-<dimension>-<YYYY-MM-DD_HH-MM-SS>-<kind>.png, the time in UTC
pub struct ExportName;

impl ExportName {
    pub fn file(world: &str, dim: i32, unix_secs: u64, kind: ExportKind) -> String {
        let (date, time) = Self::utc(unix_secs);
        format!("{}-{}-{date}_{time}-{}.png", Self::clean(world), Self::dimension(dim), kind.suffix())
    }

    // a cave layer adds its band: overworld-cave-5
    pub fn dimension(layer: i32) -> String {
        if let Some(band) = LayerIds::band(layer) {
            return format!("{}-cave-{band}", Self::dimension(LayerIds::dim(layer)));
        }
        match layer {
            0 => "overworld".to_string(),
            1 => "nether".to_string(),
            2 => "end".to_string(),
            other => format!("dim{other}"),
        }
    }

    // characters Windows does not allow in a file name, and spaces, become `_`
    fn clean(world: &str) -> String {
        let cleaned: String =
            world.chars().map(|c| if c.is_control() || " \\/:*?\"<>|".contains(c) { '_' } else { c }).collect();
        if cleaned.is_empty() { "world".to_string() } else { cleaned }
    }

    // ("YYYY-MM-DD", "HH-MM-SS"): the civil-from-days algorithm on the day count since 1970-01-01
    fn utc(unix_secs: u64) -> (String, String) {
        let days = (unix_secs / 86_400) as i64;
        let secs = unix_secs % 86_400;
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let doe = z - era * 146_097;
        let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let day = doy - (153 * mp + 2) / 5 + 1;
        let month = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = yoe + era * 400 + i64::from(month <= 2);
        (format!("{year:04}-{month:02}-{day:02}"), format!("{:02}-{:02}-{:02}", secs / 3600, secs / 60 % 60, secs % 60))
    }
}
