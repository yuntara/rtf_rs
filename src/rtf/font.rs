// use super::*;

#[derive(Clone, Debug)]
pub enum FontFamily {
    Nil,
    Roman,
    Swiss,
    Modern,
    Script,
    Decor,
    Tech,
    Bidi,
}
#[derive(Clone, Debug)]
pub struct Font {
    pub number: i32,
    pub family: FontFamily,
    pub font_name: String,
    pub alt_font_name: Option<String>,
    pub charset: Option<i32>,
    pub pitch: Option<i32>,
}
impl Font {
    pub fn new() -> Font {
        Font {
            number: 1,
            family: FontFamily::Roman,
            alt_font_name: None,
            charset: None,
            pitch: None,
            font_name: "Times New Roman".to_owned(),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct FontStyle {
    pub bold: bool,
    pub strike: bool,
    pub italic: bool,
    pub underline: bool,
    pub size: Option<i32>,
}
impl FontStyle {
    pub fn new() -> FontStyle {
        FontStyle {
            bold: false,
            strike: false,
            italic: false,
            underline: false,
            size: None,
        }
    }
}
