extern crate encoding_rs;

mod color;
mod destination;
mod document;
pub mod docx;
mod font;
mod group;
mod rtf_control;
mod style;
mod table;
mod table_border;
mod text;

use std::cell::RefCell;
use std::rc::Rc;

//use encoding_rs;
use rtf_grimoire::tokenizer::parse as parse_tokens;
use rtf_grimoire::tokenizer::Token;
use std::collections::HashMap;

pub use crate::errors::*;
use color::*;
use destination::*;
use document::*;
use font::*;
use group::*;
use style::*;
use table::*;
use table_border::*;
use text::*;

pub struct Rtf {
    tokens: Vec<Token>,
}

impl Rtf {
    fn tokenize(data: &[u8]) -> Result<Vec<Token>, Errors> {
        parse_tokens(data).map_err(|_| Errors::ParseError)
    }
    pub fn from_base64(data: &str) -> Result<Self, Errors> {
        let bytes = base64::decode(data).map_err(|e| Errors::Base64DecodeError(e))?;
        Self::from_bytes(&bytes)
    }
    pub fn from_bytes(data: &[u8]) -> Result<Self, Errors> {
        Ok(Self {
            tokens: Rtf::tokenize(data)?,
        })
    }
    pub fn get_text(
        self,
    ) -> (
        Option<Text>,
        HashMap<i32, Font>,
        HashMap<i32, StyleSheet>,
        Vec<Color>,
        Option<i32>,
    ) {
        let mut state = DocumentState::new();

        for token in self.tokens.iter().filter(|c| c != &&Token::Newline) {
            state.process_token(token);
        }
        let font_table = state.fonts;
        let stylesheets = state.stylesheets;
        let dest = (*state.destinations).borrow();
        let color_table = state.colors;
        let default_font_number = state.default_font_number;
        if let Some(dest) = dest.get("rtf") {
            debug!("Writing rtf1 content...");
            if let Destination::Text(text) = dest {
                (
                    Some(text.clone()),
                    font_table,
                    stylesheets,
                    color_table,
                    default_font_number,
                )
            } else {
                (
                    None,
                    font_table,
                    stylesheets,
                    color_table,
                    default_font_number,
                )
            }
        } else {
            (
                None,
                font_table,
                stylesheets,
                color_table,
                default_font_number,
            )
        }
    }
    pub fn into_text(self) -> String {
        let mut state = DocumentState::new();

        for token in self.tokens.iter().filter(|c| c != &&Token::Newline) {
            state.process_token(token);
        }
        let dest = (*state.destinations).borrow();
        if let Some(dest) = dest.get("rtf") {
            debug!("Writing rtf1 content...");
            if let Destination::Text(text) = dest {
                text.to_string()
            } else {
                let bytes = dest.as_bytes();
                let (cow, _used_encoding, _has_error) = encoding_rs::SHIFT_JIS.decode(&bytes);
                cow.into_owned()
            }
        } else {
            "".to_owned()
        }
    }
}
