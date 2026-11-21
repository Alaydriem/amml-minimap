use std::collections::HashMap;

use amml_api::testing::{FakeChunk, FakeWorld};
use amml_api::{BlockTypeId, ChunkLoad, ColumnSample, Rgba};

// block type ids and biome ids of the fake worlds, and builders for their chunks
pub struct FakeTerrain;

impl FakeTerrain {
    pub const STONE: BlockTypeId = BlockTypeId(1);
    pub const WATER: BlockTypeId = BlockTypeId(2);
    pub const SAND: BlockTypeId = BlockTypeId(3);
    pub const PLACEHOLDER: BlockTypeId = BlockTypeId(4);
    pub const PLAINS: u16 = 1;
    pub const OCEAN: u16 = 0;
    pub const GRASS: Rgba = Rgba { r: 127, g: 178, b: 56, a: 255 };

    pub fn names() -> HashMap<BlockTypeId, String> {
        HashMap::from([
            (Self::STONE, "minecraft:stone".to_string()),
            (Self::WATER, "minecraft:water".to_string()),
            (Self::SAND, "minecraft:sand".to_string()),
            (Self::PLACEHOLDER, "minecraft:client_request_placeholder_block".to_string()),
        ])
    }

    // a stone column at y 64, plains, with the game's stone colour
    pub fn stone() -> ColumnSample {
        ColumnSample {
            shown: Self::STONE,
            shown_color: Rgba { r: 1, g: 2, b: 3, a: 255 },
            shown_y: 64,
            height: 65,
            solid: 65,
            biome: Self::PLAINS,
            ..ColumnSample::default()
        }
    }

    pub fn chunk(load: ChunkLoad, sample: ColumnSample) -> FakeChunk {
        FakeChunk { load, samples: [sample; 256] }
    }

    // a world whose chunks at `loaded` are loaded stone, with the block and biome names
    pub fn world(loaded: &[(i32, i32)]) -> FakeWorld {
        let chunks = loaded.iter().map(|pos| (*pos, Self::chunk(ChunkLoad::Loaded, Self::stone()))).collect();
        FakeWorld {
            chunks,
            names: Self::names(),
            biome_names: HashMap::from([(Self::PLAINS, "plains".to_string()), (Self::OCEAN, "ocean".to_string())]),
            ..FakeWorld::default()
        }
    }
}
