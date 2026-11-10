// how a text anchor relates to the text box
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextAlign {
    // anchor at the top-left
    Left,
    // anchor at the top-centre
    Centre,
    // anchor at the bottom-centre
    CentreBottom,
}
