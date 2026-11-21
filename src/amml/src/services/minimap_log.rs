use amml_api::Logger;

use crate::types::ColorSource;

// the lines that show each minimap feature working, written to the mod's log through the loader. A line that could
// repeat every tick is written once per world, or once per process for the colour sources
pub struct MinimapLog {
    log: Logger,
    // a bit per colour source already logged
    sources: u8,
    chunks_logged: bool,
    biome_logged: bool,
    column_logged: bool,
}

impl MinimapLog {
    // the loaded chunk reads of one world after which `chunks read` is logged
    pub const CHUNKS_READ_LINE: u64 = 32;

    pub fn new(log: Logger) -> Self {
        Self { log, sources: 0, chunks_logged: false, biome_logged: false, column_logged: false }
    }

    pub fn logger(&self) -> Logger {
        self.log
    }

    pub fn world_entered(&mut self, dimension: i32) {
        self.chunks_logged = false;
        self.biome_logged = false;
        self.column_logged = false;
        self.log.info("minimap world entered", &[("dimension", &dimension.to_string())]);
    }

    pub fn world_named(&self, kind: &str, from_folder: bool) {
        self.log.info("minimap world named", &[("kind", kind), ("folder", if from_folder { "1" } else { "0" })]);
    }

    pub fn world_left(&self) {
        self.log.info("minimap world left", &[]);
    }

    pub fn dimension(&self, dimension: i32) {
        self.log.info("minimap dimension", &[("dimension", &dimension.to_string())]);
    }

    // `loaded`: the loaded chunks read in this world so far
    pub fn chunks_read(&mut self, loaded: u64) {
        if self.chunks_logged || loaded < Self::CHUNKS_READ_LINE {
            return;
        }
        self.chunks_logged = true;
        self.log.info("minimap chunks read", &[("loaded", &loaded.to_string())]);
    }

    pub fn colours(&mut self, source: ColorSource) {
        let bit = 1u8 << (source as u8);
        if source == ColorSource::None || self.sources & bit != 0 {
            return;
        }
        self.sources |= bit;
        self.log.info("minimap colours", &[("source", source.as_str())]);
    }

    pub fn biome_named(&mut self, id: u16, name: &str) {
        if self.biome_logged {
            return;
        }
        self.biome_logged = true;
        self.log.info("minimap biome named", &[("id", &id.to_string()), ("name", name)]);
    }

    pub fn block_change_column(&mut self, x: i32, z: i32) {
        if self.column_logged {
            return;
        }
        self.column_logged = true;
        self.log.info("minimap block change column", &[("x", &x.to_string()), ("z", &z.to_string())]);
    }
}
