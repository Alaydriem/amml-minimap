use amml_api::{Area, BlockSource, ChunkLoad, ColumnRequest, ColumnSample as GameSample, Ctx, LevelChunk};
use minimap_core::structs::chunk::{ChunkProbe, ColumnSample};
use minimap_core::structs::pos::ChunkPos;
use minimap_runtime::structs::SubmitStatus;

use crate::services::{BiomeNames, BlockNames, ColumnColors};
use crate::traits::MapSink;
use crate::types::ColorSource;

// reads loaded chunks through the loader into the probe layout the runtime takes. The walk, the game colours and
// the biome ids are the loader's; this applies the scan plan's colour table and the void overrides. Game thread
pub struct ColumnProbe {
    names: BlockNames,
    biomes: BiomeNames,
    // the loader writes one chunk's columns here; kept, so a probe allocates nothing
    samples: Box<[GameSample; 256]>,
    camera_y: i32,
    transparent_water: bool,
    // the loader serves the game's map colours
    vanilla: bool,
    // a bit per ColorSource met since the probe was made
    sources: u8,
}

impl ColumnProbe {
    pub const DEFAULT_CAMERA_Y: i32 = 320;

    pub fn new(ctx: &Ctx) -> Self {
        Self {
            names: BlockNames::new(ctx.blocks()),
            biomes: BiomeNames::new(ctx.biomes()),
            samples: Box::new([GameSample::default(); 256]),
            camera_y: Self::DEFAULT_CAMERA_Y,
            transparent_water: true,
            vanilla: ctx.supports(Area::WORLD_MAP_COLOR),
            sources: 0,
        }
    }

    // the camera height and transparent water of the latest scan plan
    pub fn configure(&mut self, camera_y: i32, transparent_water: bool) {
        self.camera_y = camera_y;
        self.transparent_water = transparent_water;
    }

    pub fn biomes(&mut self) -> &mut BiomeNames {
        &mut self.biomes
    }

    // whether a shown column has taken its colour from `source` since the probe was made
    pub fn met(&self, source: ColorSource) -> bool {
        self.sources & Self::bit(source) != 0
    }

    // fills `out` (zeroed by the caller) with the chunk at `pos`
    pub fn probe_chunk<S: MapSink>(
        &mut self,
        source: &BlockSource<'_>,
        pos: ChunkPos,
        sink: &S,
        out: &mut ChunkProbe,
    ) -> SubmitStatus {
        let Some(chunk) = source.chunk(pos.x, pos.z) else { return SubmitStatus::NotLoaded };
        match chunk.load_state() {
            Some(ChunkLoad::Loaded) => {}
            Some(ChunkLoad::TopPlaceholder) => return SubmitStatus::TopPlaceholder,
            Some(ChunkLoad::NotLoaded) | None => return SubmitStatus::NotLoaded,
        }
        let request = ColumnRequest { columns: [u64::MAX; 4], ..self.request() };
        if !self.read(&chunk, &request) {
            return SubmitStatus::NotProbed;
        }
        for index in 0..256 {
            let sample = self.samples[index];
            if sample.skipped() {
                out.surface.set_skipped(index, true);
                continue;
            }
            let column = self.convert(&sample, sink);
            out.surface.write_column(index, column);
            out.biome_ids[index] = column.biome_id;
        }
        SubmitStatus::Loaded
    }

    // one column (`index`, `z * 16 + x`) of the chunk at `pos`; `None` when the chunk is not loaded, the column was
    // not read or `index` is 256 or more
    pub fn probe_column<S: MapSink>(
        &mut self,
        source: &BlockSource<'_>,
        pos: ChunkPos,
        index: usize,
        sink: &S,
    ) -> Option<ColumnSample> {
        if index >= 256 {
            return None;
        }
        let chunk = source.chunk(pos.x, pos.z)?;
        if chunk.load_state()? != ChunkLoad::Loaded {
            return None;
        }
        let one = ColumnRequest::one((index % 16) as u8, (index / 16) as u8, self.camera_y);
        let request = ColumnRequest { columns: one.columns, ..self.request() };
        if !self.read(&chunk, &request) {
            return None;
        }
        let sample = self.samples[index];
        (!sample.skipped()).then(|| self.convert(&sample, sink))
    }

    fn request(&self) -> ColumnRequest {
        ColumnRequest {
            camera_y: self.camera_y,
            transparent_water: self.transparent_water,
            vanilla_colors: self.vanilla,
            columns: [0; 4],
        }
    }

    fn read(&mut self, chunk: &LevelChunk<'_>, request: &ColumnRequest) -> bool {
        chunk.read_columns(request, &mut self.samples).is_some()
    }

    fn convert<S: MapSink>(&mut self, sample: &GameSample, sink: &S) -> ColumnSample {
        let (column, source) = ColumnColors::convert(sample, self.vanilla, &mut self.names, &mut self.biomes, sink);
        self.sources |= Self::bit(source);
        column
    }

    fn bit(source: ColorSource) -> u8 {
        match source {
            ColorSource::Table => 1,
            ColorSource::Vanilla => 2,
            ColorSource::Fallback => 4,
            ColorSource::None => 8,
        }
    }
}
