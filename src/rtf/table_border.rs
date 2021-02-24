pub use super::*;
#[derive(Clone, Debug)]
pub enum BorderType {
    None,
    SingleThickness,
    DoubleThickness,
    Shadowed,
    Double,
    Dotted,
    Dashed,
    Hairline,
}
#[derive(Clone, Debug)]
pub struct Border {
    pub border_type: BorderType,
    pub width: usize,
}
impl Border {
    pub fn new() -> Border {
        Border {
            border_type: BorderType::None,
            width: 0,
        }
    }
}
#[derive(Clone, Debug)]
pub struct CellBorder {
    pub top: Option<Border>,
    pub left: Option<Border>,
    pub bottom: Option<Border>,
    pub right: Option<Border>,
}
impl CellBorder {
    pub fn new() -> CellBorder {
        CellBorder {
            top: None,
            left: None,
            bottom: None,
            right: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct RowBorder {
    pub top: Option<Border>,
    pub left: Option<Border>,
    pub bottom: Option<Border>,
    pub right: Option<Border>,
    pub horizontal: Option<Border>,
    pub vertical: Option<Border>,
}
impl RowBorder {
    pub fn new() -> RowBorder {
        RowBorder {
            top: None,
            left: None,
            bottom: None,
            right: None,
            horizontal: None,
            vertical: None,
        }
    }
}

#[derive(Clone)]
pub enum BorderSelect {
    RowTop,
    RowLeft,
    RowBottom,
    RowRight,
    RowHorizontal,
    RowVertical,
    CellTop,
    CellLeft,
    CellRight,
    CellBottom,
    Paragraph,
}
