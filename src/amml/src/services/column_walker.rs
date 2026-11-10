use amml_game::{Block, ChunkBlockPos, GameEntries, LevelChunk};
use minimap_core::structs::Rgba;

use crate::services::MapColorReader;
use crate::traits::MapSink;
use crate::types::{BlockKind, BlockPos, ProbeSettings, WalkedColumn};

// walks one column of a loaded chunk from the camera height down to the block the map shows. Every block read is
// inside the chunk's own height range
pub struct ColumnWalker<'w, 'e, S: MapSink> {
    chunk: &'w LevelChunk<'e>,
    entries: &'e GameEntries,
    colors: &'w MapColorReader,
    sink: &'w S,
    x: u8,
    z: u8,
    min_y: i32,
    max_y: i32,
    origin: [i32; 2],
    transparent_water: bool,
    retry: bool,
}

impl<'w, 'e, S: MapSink> ColumnWalker<'w, 'e, S> {
    // `None` when the chunk's height range cannot be read
    pub fn new(
        chunk: &'w LevelChunk<'e>,
        entries: &'e GameEntries,
        colors: &'w MapColorReader,
        sink: &'w S,
        x: u8,
        z: u8,
    ) -> Option<Self> {
        let (min_y, max_y) = (chunk.min_y()?, chunk.max_y()?);
        let [chunk_x, chunk_z] = chunk.position()?;
        Some(Self {
            chunk,
            entries,
            colors,
            sink,
            x,
            z,
            min_y,
            max_y,
            origin: [chunk_x.wrapping_mul(16), chunk_z.wrapping_mul(16)],
            transparent_water: true,
            retry: false,
        })
    }

    pub fn walk(mut self, settings: ProbeSettings) -> WalkedColumn {
        self.transparent_water = settings.transparent_water;
        let mut column = WalkedColumn::default();
        column.colour = self.pick(settings.camera_height, &mut column);
        column.retry = self.retry;
        column
    }

    fn local(&self, y: i32) -> ChunkBlockPos {
        ChunkBlockPos { x: self.x, z: self.z, y: (y - self.min_y) as i16 }
    }

    fn world_pos(&self, y: i32) -> BlockPos {
        BlockPos { x: self.origin[0] + i32::from(self.x), y, z: self.origin[1] + i32::from(self.z) }
    }

    fn block_at(&self, y: i32) -> Option<Block<'e>> {
        if y < self.min_y || y > self.max_y {
            return None;
        }
        self.chunk.block(self.local(y))
    }

    fn kind_at(&mut self, y: i32) -> BlockKind {
        let block = self.block_at(y);
        let kind = BlockKind::of(block.as_ref().and_then(|block| block.type_name()).map(|name| name.as_bytes()));
        if kind == BlockKind::Placeholder {
            self.retry = true;
        }
        kind
    }

    // a block whose material cannot be read counts as solid
    fn is_solid(&self, y: i32) -> bool {
        self.block_at(y).and_then(|block| block.is_solid()).unwrap_or(true)
    }

    fn map_colour(&self, y: i32) -> Rgba {
        let Some(block) = self.block_at(y) else { return Rgba::TRANSPARENT };
        self.colors.block_color(
            self.entries,
            self.chunk,
            &block,
            self.local(y),
            self.world_pos(y),
            self.sink.block_colors(),
        )
    }

    fn void_colour(&self) -> Rgba {
        self.colors.void_color(self.entries, self.chunk, self.local(self.min_y), self.world_pos(self.min_y), self.sink)
    }

    // transparent water: the solid colour below goes in the result, the water colour and depth beside it
    fn water(&mut self, surface_y: i32, column: &mut WalkedColumn) -> Rgba {
        column.water_colour = self.map_colour(surface_y);
        for y in (self.min_y..surface_y).rev() {
            let kind = self.kind_at(y);
            if kind != BlockKind::Water && kind != BlockKind::Air {
                column.water_depth = (surface_y - y).min(255) as u8;
                return self.map_colour(y);
            }
        }
        column.water_depth = (surface_y - self.min_y).clamp(0, 255) as u8;
        column.water_colour
    }

    fn show(&mut self, y: i32, kind: BlockKind, column: &mut WalkedColumn) -> Rgba {
        if self.transparent_water && kind == BlockKind::Water {
            return self.water(y, column);
        }
        self.map_colour(y)
    }

    // the block below the first air at or above `from`, up to `to`; the block at `to` when none is air
    fn first_below_air(&mut self, from: i32, to: i32, column: &mut WalkedColumn) -> Rgba {
        for y in from..=to {
            if self.kind_at(y) != BlockKind::Air {
                continue;
            }
            if y <= self.min_y {
                return self.void_colour();
            }
            let kind = self.kind_at(y - 1);
            return self.show(y - 1, kind, column);
        }
        let kind = self.kind_at(to);
        self.show(to, kind, column)
    }

    fn pick(&mut self, camera: i32, column: &mut WalkedColumn) -> Rgba {
        let index = usize::from(self.z) * 16 + usize::from(self.x);
        let height = i32::from(self.chunk.height(index).unwrap_or(0)) + self.min_y;
        if camera < self.min_y {
            return Rgba::TRANSPARENT;
        }
        if camera > self.max_y {
            let from = if height < self.min_y || height > self.max_y { self.max_y } else { height };
            return self.first_below_air(from, self.max_y, column);
        }
        let camera_kind = self.kind_at(camera);
        if camera_kind == BlockKind::Air {
            if height < camera {
                return self.first_below_air(height, camera, column);
            }
            for y in (self.min_y..=camera).rev() {
                let kind = self.kind_at(y);
                if kind != BlockKind::Air {
                    return self.show(y, kind, column);
                }
            }
            return self.void_colour();
        }
        if !self.is_solid(camera) {
            return self.show(camera, camera_kind, column);
        }
        // under a solid run: the first non-solid block, or the first block below the first air gap
        for y in (self.min_y..camera).rev() {
            let kind = self.kind_at(y);
            if kind == BlockKind::Air {
                for below in (self.min_y..y).rev() {
                    let below_kind = self.kind_at(below);
                    if below_kind != BlockKind::Air {
                        return self.show(below, below_kind, column);
                    }
                }
                return self.void_colour();
            }
            if !self.is_solid(y) {
                return self.show(y, kind, column);
            }
        }
        Rgba::TRANSPARENT
    }
}
