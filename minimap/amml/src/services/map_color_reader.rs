use amml_game::{Block, ChunkBlockPos, GameEntries, LevelChunk};
use minimap_core::structs::Rgba;

use crate::game::{ChunkBiomes, MapColorCalls};
use crate::traits::MapSink;
use crate::types::{BlockKind, BlockPos};

// the colour the map shows for one block. A colour from the scan plan's table wins; then the game's vanilla map
// colour when that feature is on; else a fixed colour per kind of block
pub struct MapColorReader {
    vanilla: bool,
}

impl MapColorReader {
    // the vanilla map colours of stone, water and grass, for when the game's are off or cannot be read
    pub const FALLBACK_BLOCK: Rgba = Rgba { r: 112, g: 112, b: 112, a: 255 };
    pub const FALLBACK_WATER: Rgba = Rgba { r: 64, g: 64, b: 255, a: 255 };
    pub const FALLBACK_VOID: Rgba = Rgba { r: 127, g: 178, b: 56, a: 255 };

    pub fn new(vanilla: bool) -> Self {
        Self { vanilla }
    }

    pub fn vanilla(&self) -> bool {
        self.vanilla
    }

    // the table colour of the block called `name`
    pub fn table_color(name: &[u8], colors: &[(String, u32)]) -> Option<Rgba> {
        colors.iter().find(|(entry, _)| entry.as_bytes() == name).map(|(_, colour)| Rgba::from_packed(*colour))
    }

    // 0xAARRGGBB or 0x00RRGGBB from the game's samplers, drawn opaque
    pub fn from_argb(argb: u32) -> Rgba {
        let [b, g, r, _] = argb.to_le_bytes();
        Rgba::new(r, g, b, 255)
    }

    // the colour of `block`, at `local` in `chunk` and `pos` in the world. Game thread
    pub fn block_color(
        &self,
        entries: &GameEntries,
        chunk: &LevelChunk<'_>,
        block: &Block<'_>,
        local: ChunkBlockPos,
        pos: BlockPos,
        colors: &[(String, u32)],
    ) -> Rgba {
        let name = block.type_name().map(|name| name.as_bytes());
        let kind = BlockKind::of(name);
        if kind == BlockKind::Placeholder {
            return Rgba::TRANSPARENT;
        }
        if let Some(colour) = name.and_then(|name| Self::table_color(name, colors)) {
            return colour;
        }
        let fallback = if kind == BlockKind::Water { Self::FALLBACK_WATER } else { Self::FALLBACK_BLOCK };
        if !self.vanilla {
            return fallback;
        }
        Self::vanilla_color(entries, chunk, block, local, pos, kind).unwrap_or(fallback)
    }

    // the game's map colour: water from its biome, else the map colour component tinted by its biome policy, else
    // the block type's colour
    #[allow(unsafe_code)]
    fn vanilla_color(
        entries: &GameEntries,
        chunk: &LevelChunk<'_>,
        block: &Block<'_>,
        local: ChunkBlockPos,
        pos: BlockPos,
        kind: BlockKind,
    ) -> Option<Rgba> {
        let calls = MapColorCalls::new(entries);
        if kind == BlockKind::Water {
            let biome = ChunkBiomes::new(entries).at(chunk, local)?;
            // SAFETY: the game returned a live biome of this chunk
            return unsafe { calls.water_color(biome.as_ptr(), &pos) }.map(Self::from_argb);
        }
        let dimension = chunk.dimension()?;
        // SAFETY: the chunk's dimension is live while the chunk is
        let source = unsafe { calls.main_source(dimension.as_ptr()) }?;
        // SAFETY: the block is a live block of this chunk
        if let Some(component) = unsafe { calls.component(block.as_ptr()) } {
            // SAFETY: the game returned a live BlockSource for the dimension
            let tint = unsafe { calls.policy_tint(component.tint, source, &pos) }?;
            return Some(component.base.tinted(tint).to_rgba());
        }
        // SAFETY: the block and the source are live, as above
        unsafe { calls.block_type_color(block.as_ptr(), source, &pos) }.map(|colour| colour.to_rgba())
    }

    // the colour of a column with no blocks: the biome's terrain.biomeColorOverrides entry, else the game's map
    // grass colour, else FALLBACK_VOID. `local` is the column at the chunk's lowest block, `pos` the same in the
    // world. Game thread
    #[allow(unsafe_code)]
    pub fn void_color<S: MapSink>(
        &self,
        entries: &GameEntries,
        chunk: &LevelChunk<'_>,
        local: ChunkBlockPos,
        pos: BlockPos,
        sink: &S,
    ) -> Rgba {
        let Some(biome) = ChunkBiomes::new(entries).at(chunk, local) else { return Self::FALLBACK_VOID };
        let grass = if self.vanilla {
            // SAFETY: the game returned a live biome of this chunk
            unsafe { MapColorCalls::new(entries).grass_color(biome.as_ptr(), &pos) }.map(Self::from_argb)
        } else {
            None
        };
        let fallback = grass.unwrap_or(Self::FALLBACK_VOID);
        match biome.name().to_str().ok() {
            Some(name) => Rgba::from_packed(sink.resolve_void_color(name, fallback.packed())),
            None => fallback,
        }
    }
}
