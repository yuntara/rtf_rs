use super::*;

#[derive(Clone, Debug)]
pub struct StyleSheet {
    pub number: i32,
    pub name: String,
    pub font_style: Option<FontStyle>,
    pub para_style: Option<ParagraphStyle>,
}
impl std::default::Default for StyleSheet {
    fn default() -> StyleSheet {
        StyleSheet {
            number: 0,
            name: "Default".to_owned(),
            font_style: None,
            para_style: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct ParagraphStyle {
    pub align: Option<Align>,
    pub first_indent: Option<i32>,
    pub left_indent: Option<i32>,
    pub right_indent: Option<i32>,
}

impl std::default::Default for ParagraphStyle {
    fn default() -> Self {
        Self {
            align: None,
            first_indent: None,
            left_indent: None,
            right_indent: None,
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Align {
    Left,
    Right,
    Justify,
    Center,
}
