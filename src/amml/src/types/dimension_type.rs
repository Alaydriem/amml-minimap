// the game's DimensionType struct: one int. A member function returns it through a hidden pointer
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DimensionType {
    pub value: i32,
}
