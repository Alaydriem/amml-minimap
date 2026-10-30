use std::ffi::c_void;

use amml_msvc::MsvcString;

use amml_game::MaterialBoolsLayout;

use super::{FakeBlock, FakeBlockType, FakeHeap};

// the blocks of a fake world, by name; every name is made on FakeHeap and freed when the palette drops
pub struct FakePalette {
    solid: Box<MaterialBoolsLayout>,
    liquid: Box<MaterialBoolsLayout>,
    // boxed, so each block keeps pointing at its type while the list grows
    #[allow(clippy::vec_box)]
    types: Vec<Box<FakeBlockType>>,
    blocks: Vec<(String, Box<FakeBlock>)>,
}

impl FakePalette {
    pub const AIR: &'static str = "minecraft:air";
    pub const STONE: &'static str = "minecraft:stone";
    pub const LAVA: &'static str = "minecraft:lava";
    pub const WATER: &'static str = "minecraft:water";
    pub const PLACEHOLDER: &'static str = "minecraft:client_request_placeholder_block";

    pub fn new() -> Self {
        let mut palette = Self {
            solid: Box::new(Self::material(1)),
            liquid: Box::new(Self::material(0)),
            types: Vec::new(),
            blocks: Vec::new(),
        };
        for name in [Self::AIR, Self::STONE, Self::LAVA, Self::WATER, Self::PLACEHOLDER] {
            palette.add(name);
        }
        palette
    }

    fn material(solid: u8) -> MaterialBoolsLayout {
        MaterialBoolsLayout {
            material_type: 0,
            never_buildable: 0,
            liquid: 0,
            blocks_motion: 0,
            blocks_precipitation: 0,
            solid,
            super_hot: 0,
        }
    }

    fn add(&mut self, name: &str) {
        let solid = name == Self::STONE || name == Self::PLACEHOLDER;
        let material: *const MaterialBoolsLayout = if solid { &*self.solid } else { &*self.liquid };
        // SAFETY: FakeHeap's functions are a valid allocate and free pair
        let text = unsafe { MsvcString::new_in(&FakeHeap::msvc(), name) }.unwrap();
        let mut block_type = FakeBlockType::new(text, material);
        let block = Box::new(FakeBlock::new(&raw mut *block_type));
        self.types.push(block_type);
        self.blocks.push((name.to_string(), block));
    }

    pub fn block(&self, name: &str) -> *mut c_void {
        let (_, block) = self.blocks.iter().find(|(entry, _)| entry == name).unwrap();
        (&raw const **block).cast_mut().cast()
    }
}

impl Drop for FakePalette {
    fn drop(&mut self) {
        for block_type in &mut self.types {
            // SAFETY: the name was made by `new_in` on FakeHeap and is not used again
            unsafe { block_type.layout.full_name.text.release(&FakeHeap::msvc()) };
        }
    }
}

impl Default for FakePalette {
    fn default() -> Self {
        Self::new()
    }
}
