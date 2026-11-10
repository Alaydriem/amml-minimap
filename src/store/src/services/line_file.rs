use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use crate::errors::StoreError;

// a text file written one line at a time (telemetry)
pub struct LineFile;

impl LineFile {
    pub fn append(path: &Path, line: &str, truncate: bool) -> Result<(), StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let mut file = if truncate {
            OpenOptions::new().create(true).write(true).truncate(true).open(path)?
        } else {
            OpenOptions::new().create(true).append(true).open(path)?
        };
        file.write_all(line.as_bytes())?;
        file.write_all(b"\n")?;
        Ok(())
    }
}
