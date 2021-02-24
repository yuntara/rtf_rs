use super::*;

#[derive(Clone)]
pub struct DocumentState {
    pub destinations: Rc<RefCell<HashMap<String, Destination>>>,
    pub group_stack: Vec<GroupState>,
    pub buffer: Vec<u8>,
    pub fonts: HashMap<i32, Font>,
    pub stylesheets: HashMap<i32, StyleSheet>,
}
impl DocumentState {
    pub fn new() -> Self {
        Self {
            destinations: Rc::new(RefCell::new(HashMap::new())),
            group_stack: Vec::new(),
            buffer: vec![],
            fonts: HashMap::new(),
            stylesheets: HashMap::new(),
        }
    }

    pub fn do_control_bin(&mut self, _data: &[u8], _word_is_optional: bool) {
        //println!("not support do conotrol bin");
        // We don't support handling control bins
    }

    pub fn do_control_symbol(&mut self, symbol: char, word_is_optional: bool) {
        let mut sym_bytes = [0; 4];
        let sym_str = symbol.encode_utf8(&mut sym_bytes);
        if let Some(mut group_state) = self.get_last_group_mut() {
            if let Some(symbol_handler) = rtf_control::SYMBOLS.get(sym_str) {
                symbol_handler(&mut group_state, sym_str, None);
            } else if word_is_optional {
                warn!("Skipping optional unsupported control word \\{}", symbol);
            } else {
                warn!("Unsupported/illegal control symbol \\{} (writing to document anyway)", symbol);
                self.write_to_current_destination(format!("{}", symbol).as_bytes());
            }
        } else {
            warn!(
                "Document format error: Control symbol found outside of any document group: '\\{}'",
                symbol
            );
        }
    }

    pub fn do_control_word(&mut self, name: &str, arg: Option<i32>, word_is_optional: bool) {
        if let Some(mut group_state) = self.get_last_group_mut() {
            if let Some(dest_handler) = rtf_control::DESTINATIONS.get(name) {
                dest_handler(&mut group_state, name, arg);
            } else if let Some(symbol_handler) = rtf_control::SYMBOLS.get(name) {
                symbol_handler(&mut group_state, name, arg);
            } else if let Some(value_handler) = rtf_control::VALUES.get(name) {
                value_handler(&mut group_state, name, arg);
            } else if let Some(flag_handler) = rtf_control::FLAGS.get(name) {
                flag_handler(&mut group_state, name, arg);
            } else if let Some(toggle_handler) = rtf_control::TOGGLES.get(name) {
                toggle_handler(&mut group_state, name, arg);
            } else if word_is_optional {
                warn!("Skipping optional unsupported control word \\{}", name);
            } else {
                warn!("Unsupported/illegal control word \\{}", name);
            }
        } else {
            warn!(
                "Document format error: Control word found outside of any document group: '\\{}'",
                name
            );
        }
    }

    pub fn write_to_current_destination(&mut self, bytes: &[u8]) {
        if let Some(group) = self.get_last_group_mut() {
            group.write(bytes);
        } else {
            // it is a fundamental document formatting error for text to appear outside of the {\rtf1 } group
            warn!(
                "Document format error: Document text found outside of any document group: '{:?}'",
                bytes
            );
        }
    }

    pub fn start_group(&mut self) {
        if let Some(last_group) = self.get_last_group_mut() {
            last_group.flush();
        }
        if let Some(last_group) = self.get_last_group().cloned() {
            //println!("start group{{");
            self.group_stack.push(last_group.clone());
        } else {
            debug!("Creating initial group...");
            self.group_stack.push(GroupState::new(self.destinations.clone()));
        }
    }
    pub fn process_font(&mut self, group: &GroupState) {
        let number = group.values.get("f").unwrap_or(&Some(1)).unwrap_or(1);
        let charset = group.values.get("charset").unwrap_or(&None).clone();
        let mut dests = self.destinations.borrow_mut();
        let tbl = dests.get_mut("fonttbl").expect("font table not exist");

        if let Destination::Text(text) = tbl {
            let mut font_name = text.to_string();
            // remove `;`
            font_name.pop();
            text.clear();
            //println!("font {:?}", font_name);
            let font = Font {
                number,
                font_name,
                charset,
                family: group.get_font_family(),
                alt_font_name: None,
                pitch: None,
            };

            self.fonts.insert(number, font);
        }
    }
    pub fn process_stylesheet(&mut self, group: &GroupState) {
        let number = group.values.get("s").unwrap_or(&Some(0)).unwrap_or(0);
        let mut dests = self.destinations.borrow_mut();
        let tbl = dests.get_mut("fonttbl").expect("font table not exist");

        if let Destination::Text(text) = tbl {
            let mut style_name = text.to_string();
            // remove `;`
            style_name.pop();
            text.clear();
            //println!("font {:?}", font_name);
            let stylesheet = StyleSheet {
                number,
                name: style_name,
                font_style: group.get_cur_style(),
                para_style: group.get_cur_para_style(),
            };

            self.stylesheets.insert(number, stylesheet);
        }
    }
    pub fn process_group(&mut self, group: &GroupState) {
        let dest_name = group.get_destination_name();
        if let Some(dest_name) = dest_name {
            match dest_name.as_str() {
                "fonttbl" => self.process_font(group),
                "stylesheet" => self.process_stylesheet(group),

                _ => {}
            };
        }
    }
    pub fn end_group(&mut self) {
        if let Some(mut group) = self.group_stack.pop() {
            group.flush();
            self.process_group(&group);
        // TODO: destination-folding support (tables, etc)
        } else {
            warn!("Document format error: End group count exceeds number start groups");
        }
    }

    pub fn get_last_group_mut(&mut self) -> Option<&mut GroupState> {
        self.group_stack.last_mut()
    }

    pub fn get_last_group(&self) -> Option<&GroupState> {
        self.group_stack.last()
    }
    fn write_buffer(&mut self, bytes: &Vec<u8>) {
        // let mut buf = bytes.clone();
        self.buffer.extend(bytes);
    }
    fn flush_buffer(&mut self) {
        let buffer = self.buffer.clone();

        if !buffer.is_empty() {
            self.write_to_current_destination(&buffer);

            self.buffer.clear();
        }
    }
    pub fn process_token(&mut self, token: &Token) {
        let word_is_optional = self
            .get_last_group_mut()
            .map(|group| group.get_and_clear_ignore_next_control())
            .unwrap_or(false);

        // Update state for this token
        if let Token::Text(bytes) = token {
            self.write_buffer(bytes);
        } else {
            self.flush_buffer();
            match token {
                Token::ControlSymbol(c) => self.do_control_symbol(*c, word_is_optional),
                Token::ControlWord { name, arg } => self.do_control_word(name, *arg, word_is_optional),
                Token::ControlBin(data) => self.do_control_bin(data, word_is_optional),

                Token::StartGroup => self.start_group(),
                Token::EndGroup => self.end_group(),
                _ => (),
            }
        }
    }
    /*
    pub fn get_font_table(&mut self) -> String {
        let dests = self.destinations.borrow();
        let keys = dests.keys();
        let strs: Vec<String> = keys.into_iter().map(|k| k.to_owned()).collect();
        println!("keys {}", strs.join(", "));
        let tbl = dests.get("fonttbl").expect("font table not exist");

        if let Destination::Text(text) = tbl {
            text.to_string()
        } else {
            "".to_owned()
        }
    } */
}
