use std::fs;
use std::io::Write;
use std::path::Path;

use crate::errors::StoreError;

// a small JSON file, read whole and replaced whole
pub struct JsonFile;

impl JsonFile {
    pub fn read(path: &Path) -> Result<Option<String>, StoreError> {
        match fs::read_to_string(path) {
            Ok(text) => Ok(Some(text)),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    // the text goes to a temporary file first, so a crash never leaves half a file
    pub fn write(path: &Path, json: &str) -> Result<(), StoreError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        {
            let mut file = fs::File::create(&tmp)?;
            file.write_all(json.as_bytes())?;
            file.sync_all()?;
        }
        fs::rename(&tmp, path)?;
        Ok(())
    }
}
