use super::*;
#[derive(Clone, Debug)]
pub enum Destination {
    Text(Text),
    Bytes(Vec<u8>),
}
impl Destination {
    pub fn as_bytes(&self) -> Vec<u8> {
        match self {
            Destination::Text(text) => text
                .pages
                .iter()
                .flat_map(|page| page.sections.iter())
                .flat_map(|section| section.paras.iter())
                .flat_map(|paras| paras.lines.iter())
                .flat_map(|line| line.bytes.iter())
                .cloned()
                .collect(),
            Destination::Bytes(bytes) => bytes.clone(),
        }
    }
    pub fn new_page(&mut self) {
        if let Destination::Text(text) = self {
            text.pages.push(Page::new());
        }
    }
    pub fn new_section(&mut self) {
        if let Destination::Text(text) = self {
            text.last_page().sections.push(Section::new())
        }
    }
    pub fn new_paragraph(&mut self, follow_table: bool) {
        if let Destination::Text(text) = self {
            text.new_paragraph(follow_table)
        }
    }
    pub fn new_line(&mut self) {
        if let Destination::Text(text) = self {
            text.new_line()
        }
    }

    pub fn append_text(
        &mut self,
        new_bytes: &[u8],
        font: i32,
        style: Option<FontStyle>,
        para_style: Option<ParagraphStyle>,
        stylesheet: Option<i32>,
        in_table: bool,
        encoding: &'static encoding_rs::Encoding,
    ) {
        if new_bytes.len() == 0 {
            return;
        }
        if let Destination::Text(text) = self {
            if !in_table {
                text.last_or_new_paragraph(stylesheet, para_style, in_table);
            }

            {
                let base_encoding = text.encoding;
                let line = text.last_or_new_line(font, style);
                line.bytes.extend(new_bytes);
                if base_encoding != Some(encoding) {
                    line.encoding = Some(encoding);
                }
            }
        } else {
            panic!("Programmer error: attempting to add text to a byte destination");
        }
    }

    pub fn append_bytes(&mut self, new_bytes: &[u8]) {
        if let Destination::Bytes(bytes) = self {
            bytes.extend(new_bytes);
        } else {
            panic!("Programmer error: attempting to add bytes to a text destination");
        }
    }
}
