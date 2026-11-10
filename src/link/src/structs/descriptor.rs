// the descriptor the mod registers with the manager: id, claims, keybinds and the settings schema. The file ships
// beside the AMML mod crate that registers it
pub struct Descriptor;

impl Descriptor {
    pub const TEXT: &'static str = include_str!("../../../amml/descriptor.hcl");
}
