extern crate encoding_rs;

mod destination;
mod document;
mod font;
mod group;
mod rtf_control;
mod style;
mod table;
mod table_border;
mod text;

use std::rc::Rc;
use std::{cell::RefCell, io::SeekFrom};

//use encoding_rs;
use rtf_grimoire::tokenizer::parse as parse_tokens;
use rtf_grimoire::tokenizer::Token;
use std::collections::HashMap;

pub use crate::errors::*;
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
    pub fn get_text(self) -> (Option<Text>, HashMap<i32, Font>, HashMap<i32, StyleSheet>) {
        let mut state = DocumentState::new();

        for token in self.tokens.iter().filter(|c| c != &&Token::Newline) {
            state.process_token(token);
        }
        let font_table = state.fonts;
        let stylesheets = state.stylesheets;
        let dest = (*state.destinations).borrow();

        if let Some(dest) = dest.get("rtf") {
            debug!("Writing rtf1 content...");
            if let Destination::Text(text) = dest {
                (Some(text.clone()), font_table, stylesheets)
            } else {
                (None, font_table, stylesheets)
            }
        } else {
            (None, font_table, stylesheets)
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
    pub fn into_docx_base64(self) -> Result<String, Errors> {
        let docx = self.into_docx()?;
        Ok(base64::encode(docx))
    }
    pub fn into_docx(self) -> Result<Vec<u8>, Errors> {
        impl std::convert::Into<AlignmentType> for Align {
            fn into(self) -> AlignmentType {
                match self {
                    Align::Left => AlignmentType::Left,
                    Align::Justify => AlignmentType::Justified,
                    Align::Center => AlignmentType::Center,
                    Align::Right => AlignmentType::Right,
                }
            }
        }
        impl std::convert::Into<docx_rs::BorderType> for table_border::BorderType {
            fn into(self) -> docx_rs::BorderType {
                match self {
                    table_border::BorderType::SingleThickness => docx_rs::BorderType::Single,
                    table_border::BorderType::DoubleThickness => docx_rs::BorderType::Double,
                    table_border::BorderType::Shadowed => docx_rs::BorderType::Single,
                    table_border::BorderType::Double => docx_rs::BorderType::Double,
                    table_border::BorderType::Dotted => docx_rs::BorderType::Dotted,
                    table_border::BorderType::Dashed => docx_rs::BorderType::Dashed,
                    table_border::BorderType::Hairline => docx_rs::BorderType::Single,
                    table_border::BorderType::None => docx_rs::BorderType::None,
                    //_ => docx_rs::BorderType::Single,
                }
            }
        }
        impl std::convert::Into<docx_rs::TableBorders> for RowBorder {
            fn into(self) -> TableBorders {
                let mut borders = TableBorders::new();
                if let Some(top) = self.top {
                    let mut b = TableBorder::new(BorderPosition::Top);
                    b.border_type = top.border_type.into();
                    b.size = top.width;
                    borders = borders.set(b);
                }
                if let Some(left) = self.left {
                    let mut b = TableBorder::new(BorderPosition::Left);
                    b.border_type = left.border_type.into();
                    b.size = left.width;
                    borders = borders.set(b);
                }
                if let Some(right) = self.right {
                    let mut b = TableBorder::new(BorderPosition::Right);
                    b.border_type = right.border_type.into();
                    b.size = right.width;

                    borders = borders.set(b);
                }
                if let Some(bottom) = self.bottom {
                    let mut b = TableBorder::new(BorderPosition::Bottom);
                    b.border_type = bottom.border_type.into();
                    b.size = bottom.width;
                    borders = borders.set(b);
                }
                if let Some(vertical) = self.vertical {
                    let mut b = TableBorder::new(BorderPosition::InsideV);
                    b.border_type = vertical.border_type.into();
                    b.size = vertical.width;
                    borders = borders.set(b);
                }
                if let Some(horizontal) = self.horizontal {
                    let mut b = TableBorder::new(BorderPosition::InsideH);
                    b.border_type = horizontal.border_type.into();
                    b.size = horizontal.width;
                    borders = borders.set(b);
                }
                borders
            }
        }
        impl std::convert::Into<docx_rs::TableCellBorders> for CellBorder {
            fn into(self) -> TableCellBorders {
                let mut borders = TableCellBorders::new();
                if let Some(top) = self.top {
                    let mut b = TableCellBorder::new(BorderPosition::Top);
                    b.border_type = top.border_type.into();
                    b.size = top.width;
                    borders = borders.set(b);
                }
                if let Some(left) = self.left {
                    let mut b = TableCellBorder::new(BorderPosition::Left);
                    b.border_type = left.border_type.into();
                    b.size = left.width;
                    borders = borders.set(b);
                }
                if let Some(right) = self.right {
                    let mut b = TableCellBorder::new(BorderPosition::Right);
                    b.border_type = right.border_type.into();
                    b.size = right.width;

                    borders = borders.set(b);
                }
                if let Some(bottom) = self.bottom {
                    let mut b = TableCellBorder::new(BorderPosition::Bottom);
                    b.border_type = bottom.border_type.into();
                    b.size = bottom.width;
                    borders = borders.set(b);
                }
                borders
            }
        }
        fn make_run(
            line: &Line,
            font_table: &HashMap<i32, font::Font>,
            encoding: Option<&'static encoding_rs::Encoding>,
            stylesheet_font_style: &FontStyle,
        ) -> Run {
            let mut run = Run::new();
            let text = crate::rtf::Text::decode_line(encoding, &line);
            /* println!("{:?}", line.bytes);
            println!("{}", text);
            println!("{:?}", encoding);*/
            run = run.add_text(text);
            if let Some(font) = line.font {
                if let Some(font) = font_table.get(&font) {
                    let run_font = RunFonts::new().east_asia(font.font_name.clone());
                    run = run.fonts(run_font);
                }
            }
            if let Some(style) = line.style.as_ref() {
                if style.bold || stylesheet_font_style.bold {
                    run = run.bold()
                }
                if style.italic || stylesheet_font_style.italic {
                    run = run.italic()
                }
                /*if style.underline {
                    run = run.underline()
                }*/
                if let Some(size) = style.size {
                    run = run.size(size as usize);
                } else if let Some(size) = stylesheet_font_style.size {
                    run = run.size(size as usize);
                }
            }

            run
        }
        use docx_rs::*;
        use std::io::{Cursor, Read, Seek};
        let mut docx = docx_rs::Docx::new();
        let mut cursor = Cursor::new(Vec::new());

        let (text, font_table, stylesheets) = self.get_text();
        let default_stylesheet = StyleSheet::default();
        let default_font = FontStyle::new();
        let default_para_style = style::ParagraphStyle::default();
        if let Some(text) = text {
            for page in text.pages {
                //let mut i = 0;
                for section in page.sections {
                    for para in section.paras {
                        let stylesheet = stylesheets
                            .get(&para.stylesheet.unwrap_or(0))
                            .unwrap_or(&default_stylesheet);
                        let stylesheet_para = stylesheet
                            .para_style
                            .as_ref()
                            .unwrap_or(&default_para_style);
                        let para_style = para.style.as_ref().unwrap_or(&stylesheet_para);
                        let align = para_style
                            .align
                            .as_ref()
                            .or_else(|| stylesheet_para.align.as_ref());

                        let first_indent = para_style
                            .first_indent
                            .as_ref()
                            .or(stylesheet_para.first_indent.as_ref());
                        let special_indent =
                            first_indent.map(|indent| SpecialIndentType::FirstLine(indent.clone()));

                        let stylesheet_font_style =
                            stylesheet.font_style.as_ref().unwrap_or(&default_font);

                        if let Some(table) = para.table {
                            let mut rows: Vec<docx_rs::TableRow> = vec![];
                            let mut border = None;
                            //println!("{:?}", table);
                            for rtf_row in table.rows {
                                if rtf_row.border.is_some() {
                                    border = rtf_row.border.clone();
                                }
                                let mut left = Some(9);
                                //println!("{:?}", rtf_row);
                                if rtf_row.is_empty() {
                                    continue;
                                }
                                let mut cells: Vec<docx_rs::TableCell> = vec![];
                                for rtf_cell in rtf_row.cells {
                                    //println!("{:?}", rtf_cell.opts);
                                    if rtf_cell.is_empty() {
                                        continue;
                                    }

                                    let width = if let Some(left) = left {
                                        rtf_cell.opts.right.map(|r| r - left)
                                    } else {
                                        None
                                    };

                                    let mut cell = docx_rs::TableCell::new();
                                    if let Some(border) = rtf_cell.opts.border {
                                        cell = cell.set_borders(border.into());
                                    } else {
                                    }
                                    for para in rtf_cell.paras {
                                        let mut p = Paragraph::new();
                                        if let Some(align) = align {
                                            p = p.align(align.clone().into());
                                        }
                                        p = p.indent(
                                            para_style.left_indent.or(stylesheet_para.left_indent),
                                            special_indent,
                                            para_style
                                                .right_indent
                                                .or(stylesheet_para.right_indent),
                                            None,
                                        );
                                        for line in para.lines {
                                            let run = make_run(
                                                &line,
                                                &font_table,
                                                line.encoding.or(text.encoding),
                                                &stylesheet_font_style,
                                            );
                                            p = p.add_run(run);
                                        }
                                        cell = cell.add_paragraph(p);
                                    }
                                    if let Some(width) = width {
                                        cell = cell.width(width, WidthType::Auto);
                                    } else {
                                        left = None;
                                    }
                                    cells.push(cell);
                                }
                                let row = docx_rs::TableRow::new(cells);
                                rows.push(row)
                            }

                            let mut table = docx_rs::Table::new(rows);
                            if let Some(border) = border {
                                table = table.set_borders(border.into());
                            }
                            docx = docx.add_table(table);
                        } else {
                            let mut p = Paragraph::new();
                            if let Some(align) = align {
                                p = p.align(align.clone().into());
                            }
                            p = p.indent(
                                para_style.left_indent.or(stylesheet_para.left_indent),
                                special_indent,
                                para_style.right_indent.or(stylesheet_para.right_indent),
                                None,
                            );
                            for line in para.lines {
                                let run = make_run(
                                    &line,
                                    &font_table,
                                    line.encoding.or(text.encoding),
                                    &stylesheet_font_style,
                                );
                                p = p.add_run(run);
                            }

                            docx = docx.add_paragraph(p);
                        }
                    }
                }
                let mut p = Paragraph::new();
                let mut run = Run::new();
                run = run.add_break(BreakType::Page);
                p = p.add_run(run);
                docx = docx.add_paragraph(p);
            }
        }
        let zip = docx.build().pack(&mut cursor);
        if zip.is_err() {
            return Err(Errors::DocxBuildError);
        }
        // TODO: add error handler
        if let Err(_) = cursor.seek(SeekFrom::Start(0)) {
            return Err(Errors::DocxBuildError);
        }
        let mut out = Vec::new();
        if let Err(_) = cursor.read_to_end(&mut out) {
            return Err(Errors::DocxBuildError);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_yaml_snapshot;
    #[test]
    fn rtf_test_1() {
        let bytes = include_bytes!("./test/mocks/helloworld.rtf");
        let rtf = super::Rtf::from_bytes(bytes).expect("must parse");
        assert_yaml_snapshot!(rtf.into_docx().unwrap());
    }
}
