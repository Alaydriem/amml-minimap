use amml_game::{BlockSource, GameEntries, LevelChunk};
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::SubmitStatus;

use crate::services::{BiomeReader, ColumnWalker, MapColorReader};
use crate::traits::MapSink;
use crate::types::ProbeSettings;

// reads loaded game chunks into the probe layout the runtime takes; holds nothing beyond the plan's settings
pub struct ColumnProbe {
    colors: MapColorReader,
    biomes: BiomeReader,
    settings: ProbeSettings,
}

impl ColumnProbe {
    // `vanilla`: the game's map colours can be read
    pub fn new(vanilla: bool) -> Self {
        Self { colors: MapColorReader::new(vanilla), biomes: BiomeReader::new(), settings: ProbeSettings::default() }
    }

    // the camera height and transparent water of the latest scan plan
    pub fn configure(&mut self, settings: ProbeSettings) {
        self.settings = settings;
    }

    pub fn settings(&self) -> ProbeSettings {
        self.settings
    }

    pub fn biomes(&self) -> &BiomeReader {
        &self.biomes
    }

    // the chunk at `pos` when the game has it fully loaded. Game thread
    pub fn loaded_chunk<'e>(region: &BlockSource<'e>, pos: ChunkPos) -> Option<LevelChunk<'e>> {
        let chunk = region.chunk(pos.x, pos.z)?;
        chunk.loaded().then_some(chunk)
    }

    // fills `out` (zeroed by the caller) with the chunk at `pos`. Not loaded: nothing read past the load state.
    // No sub-chunks, or a top sub-chunk still the client's placeholder: nothing read past the sub-chunk list.
    // Game thread
    pub fn probe_chunk<S: MapSink>(
        &self,
        entries: &GameEntries,
        region: &BlockSource<'_>,
        pos: ChunkPos,
        sink: &S,
        out: &mut ChunkProbe,
    ) -> SubmitStatus {
        let Some(chunk) = Self::loaded_chunk(region, pos) else { return SubmitStatus::NotLoaded };
        let count = chunk.sub_chunk_count();
        if count == 0 {
            return SubmitStatus::TopPlaceholder;
        }
        if chunk.sub_chunk(count - 1).and_then(|top| top.is_placeholder()) != Some(false) {
            return SubmitStatus::TopPlaceholder;
        }
        let Some(min_y) = chunk.min_y() else { return SubmitStatus::NotProbed };
        for index in 0..256 {
            if Self::surface_pending(&chunk, index, count) {
                out.surface.set_skipped(index, true);
                continue;
            }
            let Some(sample) = self.sample(entries, &chunk, index, min_y, sink) else {
                out.surface.set_skipped(index, true);
                continue;
            };
            out.surface.surface[index] = sample.surface;
            out.surface.water[index] = sample.water;
            out.surface.height[index] = sample.height;
            out.surface.solid[index] = sample.solid;
            out.surface.set_retry(index, sample.retry);
            out.biome_ids[index] = sample.biome_id;
        }
        SubmitStatus::Loaded
    }

    // one column of the chunk at `pos`; `None` when the chunk is not loaded or `index >= 256`. Game thread
    pub fn probe_column<S: MapSink>(
        &self,
        entries: &GameEntries,
        source: &BlockSource<'_>,
        pos: ChunkPos,
        index: usize,
        sink: &S,
    ) -> Option<ColumnSample> {
        if index >= 256 {
            return None;
        }
        let chunk = Self::loaded_chunk(source, pos)?;
        let min_y = chunk.min_y()?;
        self.sample(entries, &chunk, index, min_y, sink)
    }

    // the chunk's biome names for the plan's requests, from the column probe just made
    pub fn name_biomes<S: MapSink>(
        &self,
        entries: &GameEntries,
        region: &BlockSource<'_>,
        pos: ChunkPos,
        probe: &ChunkProbe,
        requests: &[u16],
        sink: &mut S,
    ) {
        if requests.is_empty() {
            return;
        }
        if let Some(chunk) = Self::loaded_chunk(region, pos) {
            self.biomes.answer_chunk(entries, &chunk, &probe.biome_ids, requests, sink);
        }
    }

    fn sample<S: MapSink>(
        &self,
        entries: &GameEntries,
        chunk: &LevelChunk<'_>,
        index: usize,
        min_y: i32,
        sink: &S,
    ) -> Option<ColumnSample> {
        let (x, z) = ((index % 16) as u8, (index / 16) as u8);
        let column = ColumnWalker::new(chunk, entries, &self.colors, sink, x, z)?.walk(self.settings);
        let height = (i32::from(chunk.height(index)?) + min_y) as i16;
        let solid = (i32::from(chunk.render_height(index)?) + min_y) as i16;
        let biome = self.biomes.surface_id(entries, chunk, index);
        Some(ColumnSample::new(column.colour.packed(), column.water_word(), height, solid, column.retry, biome))
    }

    // the column's surface sub-chunk is still a placeholder: the client has not received it yet
    fn surface_pending(chunk: &LevelChunk<'_>, index: usize, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        let height = i32::from(chunk.height(index).unwrap_or(0));
        let sub = (height >> 4).min(count as i32 - 1);
        sub >= 0 && chunk.sub_chunk(sub as usize).and_then(|sub| sub.is_placeholder()) == Some(true)
    }
}
