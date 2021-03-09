use super::*;
#[derive(Clone)]
pub struct GroupState {
    pub destinations: Rc<RefCell<HashMap<String, Destination>>>,
    pub cur_destination: Option<String>,
    pub dest_encoding: Option<&'static encoding_rs::Encoding>,
    pub values: HashMap<String, Option<i32>>,
    pub opt_ignore_next_control: bool,
    pub cur_font: Option<i32>,
    pub buffer: Vec<u8>,
    pub border_select: BorderSelect,
    pub unicode_count: usize,
    pub ignore_count: usize,
    pub color_index: usize,
    pub colors: std::collections::VecDeque<Color>,
}
impl GroupState {
    pub fn new(destinations: Rc<RefCell<HashMap<String, Destination>>>) -> Self {
        Self {
            destinations,
            cur_destination: None,
            dest_encoding: None,
            values: HashMap::new(),
            opt_ignore_next_control: false,
            cur_font: None,
            buffer: vec![],
            border_select: BorderSelect::Paragraph,
            unicode_count: 0,
            ignore_count: 0,
            color_index: 0,
            colors: std::collections::VecDeque::new(),
        }
    }

    pub fn set_codepage(&mut self, cp: u16) {
        self.dest_encoding = codepage::to_encoding(cp);
    }

    pub fn get_encoding(&self) -> Option<&'static encoding_rs::Encoding> {
        self.dest_encoding
    }

    pub fn set_encoding(&mut self, encoding: Option<&'static encoding_rs::Encoding>) {
        self.dest_encoding = encoding;
    }

    pub fn set_destination(&mut self, name: &str, uses_encoding: bool) {
        self.cur_destination = Some(name.to_owned());
        let mut dest = (*self.destinations).borrow_mut();
        match dest.get(name) {
            Some(Destination::Text(text)) => {
                debug!(
                    "Switching to destination {}, with current page length {})",
                    name,
                    text.pages.len()
                );
                assert!(uses_encoding);
            }
            Some(Destination::Bytes(bytes)) => {
                debug!(
                    "Switching to destination {}, with current length {})",
                    name,
                    bytes.len()
                );
                assert!(!uses_encoding);
            }
            None => {
                if uses_encoding {
                    dest.insert(
                        name.to_string(),
                        Destination::Text(Text {
                            pages: vec![Page::new()],
                            encoding: self.get_encoding(),
                        }),
                    );
                } else {
                    dest.insert(name.to_string(), Destination::Bytes(Vec::new()));
                }
            }
        }
    }

    pub fn get_destination_name(&self) -> Option<String> {
        self.cur_destination.clone()
    }
    pub fn new_line(&mut self) {
        self.flush();
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            dest.new_line();
        }
    }
    pub fn new_section(&mut self) {
        self.flush();
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            dest.new_section();
        }
    }
    pub fn new_page(&mut self) {
        self.flush();
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            dest.new_page();
        }
    }
    pub fn new_paragraph(&mut self) {
        self.flush();
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            dest.new_paragraph(self.has_key("intbl"));
        }
    }
    pub fn get_cur_style(&self) -> Option<FontStyle> {
        let bold = self.has_key("b");
        let italic = self.has_key("i");
        let underline = self.has_key("u");
        let size = self.values.get("fs").unwrap_or(&None).clone();
        let cb: usize = self
            .values
            .get("cb")
            .unwrap_or(&Some(0))
            .unwrap_or(0)
            .clone() as usize;
        let cf: usize = self
            .values
            .get("cf")
            .unwrap_or(&Some(0))
            .unwrap_or(0)
            .clone() as usize;
        //self.destinations.get_mut("stylesheet");
        if !bold && !italic && !underline && size.is_none() {
            return None;
        }
        Some(FontStyle {
            bold,
            italic,
            strike: false,
            underline,
            size,
            foreground_color: cf,
            background_color: cb,
        })
    }
    pub fn get_cur_stylesheet(&self) -> Option<i32> {
        let stylesheet_num = self.values.get("s").unwrap_or(&None).clone();
        stylesheet_num
    }
    pub fn reset_paragraph_properies(&mut self) {
        self.values.remove("ql");
        self.values.remove("qr");
        self.values.remove("qj");
        self.values.remove("qc");
        self.values.remove("fi");
        self.values.remove("li");
        self.values.remove("ri");
        self.values.remove("intbl");
        self.values.remove("b");
        self.values.remove("u");
        self.values.remove("i");
        self.values.remove("fs");
    }
    pub fn get_cur_para_style(&self) -> Option<ParagraphStyle> {
        let align = if self.has_key("ql") {
            Some(Align::Left)
        } else if self.has_key("qr") {
            Some(Align::Right)
        } else if self.has_key("qj") {
            Some(Align::Justify)
        } else if self.has_key("qc") {
            Some(Align::Center)
        } else {
            None
        };
        let first_indent = self.values.get("fi").unwrap_or(&None).clone();
        let left_indent = self.values.get("li").unwrap_or(&None).clone();
        let right_indent = self.values.get("ri").unwrap_or(&None).clone();
        if align.is_none()
            && first_indent.is_none()
            && left_indent.is_none()
            && right_indent.is_none()
        {
            return None;
        }
        //self.destinations.get_mut("stylesheet");
        Some(ParagraphStyle {
            align,
            first_indent,
            left_indent,
            right_indent,
        })
    }
    pub fn next_color_index(&mut self) {
        self.colors.push_back(Color::default());
    }
    pub fn shift_color(&mut self) -> Option<Color> {
        self.colors.pop_front()
    }
    pub fn flush(&mut self) {
        if self.buffer.len() > 0 {
            {
                self.write(&self.buffer.clone());
            }
            self.buffer.clear();
        }
    }
    pub fn buffer(&mut self, bytes: &[u8]) {
        self.buffer.extend(bytes);
    }
    pub fn write_unicode(&mut self, value: i32) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!(
                    "Document format error: Document text found outside of any document group: '{:?}'",
                    value
                );
                return;
            }
        };
        if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            match dest {
                Destination::Text(_) => {
                    dest.append_text(
                        &vec![(value & 0xff) as u8, (value >> 8) as u8],
                        self.cur_font,
                        self.get_cur_style(),
                        self.get_cur_para_style(),
                        self.get_cur_stylesheet(),
                        self.has_key("intbl"),
                        encoding_rs::UTF_16LE,
                    );
                }
                Destination::Bytes(_) => { /* NOP */ }
            }
        }
    }
    pub fn write(&mut self, bytes: &[u8]) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!(
                    "Document format error: Document text found outside of any document group: '{:?}'",
                    bytes
                );
                return;
            }
        };
        if dest_name == "colortbl" {
            if bytes.len() == 1 && bytes.get(0) == Some(&59 /* = ';' */) {
                self.next_color_index();
            }
        } else if let Some(dest) = (*self.destinations).borrow_mut().get_mut(&dest_name) {
            match dest {
                Destination::Text(_) => {
                    if let Some(decoder) = self.dest_encoding {
                        dest.append_text(
                            &bytes[self.ignore_count..],
                            self.cur_font,
                            self.get_cur_style(),
                            self.get_cur_para_style(),
                            self.get_cur_stylesheet(),
                            self.has_key("intbl"),
                            decoder,
                        );
                    } else {
                        warn!(
                            "Writing to a text destination ({}) with no encoding set!",
                            dest_name
                        );
                    }
                }
                Destination::Bytes(_) => {
                    dest.append_bytes(&bytes[self.ignore_count..]);
                }
            }
            if self.ignore_count > 0 {
                self.ignore_count = self.ignore_count - 1;
            }
        } else {
            panic!(
                "Programming error: specified destination {} doesn't exist after verifying its existence",
                dest_name
            );
        }
    }

    pub fn set_opt_ignore_next_control(&mut self) {
        self.opt_ignore_next_control = true;
    }

    pub fn get_and_clear_ignore_next_control(&mut self) -> bool {
        let old = self.opt_ignore_next_control;
        self.opt_ignore_next_control = false;
        old
    }
    pub fn add_cell(&mut self) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(Destination::Text(text)) = (*self.destinations).borrow_mut().get_mut(&dest_name)
        {
            let last_paragraph = text.last_paragraph(false);

            if let Some(table) = last_paragraph.table.as_mut() {
                table.add_cell();
            }
        }
    }
    pub fn end_row(&mut self) {
        self.values.remove("intbl");
    }
    pub fn set_row(&mut self) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };

        if let Some(Destination::Text(text)) = (*self.destinations).borrow_mut().get_mut(&dest_name)
        {
            let last_paragraph = text.last_paragraph(false);
            if {
                // let last_paragraph = text.last_paragraph();
                last_paragraph.table.is_none()
                    && (last_paragraph.lines.len() > 1 || last_paragraph.lines[0].bytes.len() > 0)
            } {
                let mut p = Paragraph::new();
                p.table = Some(Table::new());

                text.last_section().paras.push(p);
            } else {
                let last_paragraph = text.last_paragraph(false);
                if let Some(table) = last_paragraph.table.as_mut() {
                    table.add_row();
                } else {
                    last_paragraph.table = Some(Table::new());
                }
            }
        }
    }
    pub fn set_border_type(&mut self, border_type: BorderType) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(Destination::Text(text)) = (*self.destinations).borrow_mut().get_mut(&dest_name)
        {
            text.set_border_type(self.border_select.clone(), border_type);
        }
    }
    pub fn set_border_width(&mut self, border_width: usize) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(Destination::Text(text)) = (*self.destinations).borrow_mut().get_mut(&dest_name)
        {
            text.set_border_width(self.border_select.clone(), border_width);
        }
    }
    pub fn set_cell_right(&mut self, right: usize) {
        let dest_name = match self.get_destination_name() {
            Some(name) => name.clone(),
            None => {
                warn!("Document format error: Document text found outside of any document group",);
                return;
            }
        };
        if let Some(Destination::Text(text)) = (*self.destinations).borrow_mut().get_mut(&dest_name)
        {
            text.set_cell_right(right);
        }
    }
    pub fn set_value(&mut self, name: &str, value: Option<i32>) {
        match name {
            "f" => {
                self.cur_font = Some(value.unwrap_or(1));
            }
            "pard" => {
                self.reset_paragraph_properies();
            }
            "trowd" => {
                self.values.insert("intbl".to_owned(), None);
                self.set_row();
            }
            "trbrdrt" => self.border_select = BorderSelect::RowTop,
            "trbrdrl" => self.border_select = BorderSelect::RowLeft,
            "trbrdrb" => self.border_select = BorderSelect::RowBottom,
            "trbrdrr" => self.border_select = BorderSelect::RowRight,
            "trbrdrh" => self.border_select = BorderSelect::RowHorizontal,
            "trbrdrv" => self.border_select = BorderSelect::RowVertical,
            "clbrdrt" => self.border_select = BorderSelect::CellTop,
            "clbrdrl" => self.border_select = BorderSelect::CellLeft,
            "clbrdrb" => self.border_select = BorderSelect::CellBottom,
            "clbrdrr" => self.border_select = BorderSelect::CellRight,
            "brdrs" => self.set_border_type(BorderType::SingleThickness),
            "brdrth" => self.set_border_type(BorderType::DoubleThickness),
            "brdrsh" => self.set_border_type(BorderType::Shadowed),
            "brdrdb" => self.set_border_type(BorderType::Double),
            "brdrdot" => self.set_border_type(BorderType::Dotted),
            "brdrdash" => self.set_border_type(BorderType::Dashed),
            "brdrhair" => self.set_border_type(BorderType::Hairline),
            "brdrnone" => self.set_border_type(BorderType::None),
            "brdrw" => self.set_border_width(value.unwrap_or(0) as usize),
            "cellx" => {
                if let Some(value) = value {
                    self.set_cell_right(value as usize)
                }
            }
            "uc" => {
                self.new_line();
                self.unicode_count = value.unwrap_or(0) as usize;
            }
            "u" => {
                if self.unicode_count > 0 {
                    if let Some(value) = value {
                        self.write_unicode(value)
                    }
                    self.unicode_count = self.unicode_count - 1;
                    self.ignore_count = self.ignore_count + 1;
                    if self.unicode_count == 0 {
                        self.new_line();
                    }
                }
            }
            "red" => {
                let len = self.colors.len();
                if let Some(color) = self.colors.get_mut(len - 1) {
                    if let Some(value) = value {
                        color.r = value as u8;
                    }
                }
            }
            "green" => {
                let len = self.colors.len();
                if let Some(color) = self.colors.get_mut(len - 1) {
                    if let Some(value) = value {
                        color.g = value as u8;
                    }
                }
            }
            "blue" => {
                let len = self.colors.len();
                if let Some(color) = self.colors.get_mut(len - 1) {
                    if let Some(value) = value {
                        color.b = value as u8;
                    }
                }
            }
            "b" => {
                if let Some(n) = value {
                    if n == 0 {
                        self.new_line();
                        self.values.remove("b");
                        return;
                    }
                }
            }
            "u" => {
                if let Some(n) = value {
                    if n == 0 {
                        self.new_line();
                        self.values.remove("u");

                        return;
                    }
                }
            }
            "i" => {
                if let Some(n) = value {
                    if n == 0 {
                        self.new_line();
                        self.values.remove("i");

                        return;
                    }
                }
            }

            _ => {}
        };
        self.values.insert(name.to_string(), value);
    }
    pub fn has_key(&self, k: &str) -> bool {
        self.values.contains_key(k)
    }
    pub fn get_font_family(&self) -> FontFamily {
        if self.has_key("fnil") {
            FontFamily::Nil
        } else if self.has_key("froman") {
            FontFamily::Roman
        } else if self.has_key("fswiss") {
            FontFamily::Swiss
        } else if self.has_key("fmodern") {
            FontFamily::Modern
        } else if self.has_key("fscript") {
            FontFamily::Script
        } else if self.has_key("fdecor") {
            FontFamily::Decor
        } else if self.has_key("ftech") {
            FontFamily::Tech
        } else if self.has_key("fbidi") {
            FontFamily::Bidi
        } else {
            FontFamily::Nil
        }
    }
}
