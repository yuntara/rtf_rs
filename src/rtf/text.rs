use super::*;

#[derive(Clone, Debug)]
pub struct Page {
    pub sections: Vec<Section>,
}
impl Page {
    pub fn new() -> Page {
        Page {
            sections: vec![Section::new()],
        }
    }
}
#[derive(Clone, Debug)]
pub struct Section {
    pub paras: Vec<Paragraph>,
}
impl Section {
    pub fn new() -> Section {
        Section {
            paras: vec![Paragraph::new()],
        }
    }
}

#[derive(Clone, Debug)]
pub struct Paragraph {
    pub lines: Vec<Line>,
    pub table: Option<Table>,
    pub stylesheet: Option<i32>,
    pub style: Option<ParagraphStyle>,
}
impl Paragraph {
    pub fn new() -> Paragraph {
        Paragraph {
            lines: vec![Line::new()],
            stylesheet: None,
            style: None,
            table: None,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.lines.len() == 1 && self.lines[0].bytes.len() == 0
    }
}
#[derive(Clone, Debug)]
pub struct Line {
    pub bytes: Vec<u8>,
    pub font: Option<i32>,
    pub style: Option<FontStyle>,
    pub encoding: Option<&'static encoding_rs::Encoding>,
}
impl Line {
    pub fn new() -> Line {
        Line {
            bytes: vec![],
            font: None,
            style: None,
            encoding: None,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Text {
    pub pages: Vec<Page>,
    pub encoding: Option<&'static encoding_rs::Encoding>,
}
impl std::string::ToString for Text {
    fn to_string(&self) -> String {
        let bytes: Vec<u8> = self
            .pages
            .iter()
            .flat_map(|page| page.sections.iter())
            .flat_map(|section| section.paras.iter())
            .flat_map(|paras| paras.lines.iter())
            .flat_map(|line| line.bytes.iter())
            .cloned()
            .collect();
        if let Some(decoder) = self.encoding {
            decoder.decode(&bytes).0.into_owned()
        } else {
            encoding_rs::SHIFT_JIS.decode(&bytes).0.into_owned()
        }
    }
}
impl Text {
    pub fn new() -> Text {
        Text {
            pages: vec![Page::new()],
            encoding: Some(encoding_rs::SHIFT_JIS),
        }
    }
    pub fn last_page(&mut self) -> &mut Page {
        self.pages.last_mut().expect("must exist page")
    }
    pub fn last_section(&mut self) -> &mut Section {
        self.last_page().sections.last_mut().expect("must exist section")
    }
    pub fn last_paragraph(&mut self, follow_table: bool) -> &mut Paragraph {
        let lp = self.last_section().paras.last_mut().expect("must exist paragraph");
        if lp.table.is_some() && follow_table {
            //println!("last para is table");
            let table = lp.table.as_mut().unwrap();
            table.last_cell().paras.last_mut().expect("must exist para")
        } else {
            lp
        }
    }
    pub fn last_line(&mut self) -> &mut Line {
        let p = self.last_paragraph(false);
        if let Some(table) = p.table.as_mut() {
            table
                .last_cell()
                .paras
                .last_mut()
                .expect("must exist para")
                .lines
                .last_mut()
                .expect("must exist line")
        } else {
            p.lines.last_mut().expect("must exist line")
        }
    }
    pub fn new_line(&mut self) {
        let p = self.last_paragraph(true);
        if let Some(table) = p.table.as_mut() {
            table
                .last_cell()
                .paras
                .last_mut()
                .expect("must exist para")
                .lines
                .push(Line::new());
        } else {
            p.lines.push(Line::new());
        }
    }
    pub fn decode_line(encoding: Option<&'static encoding_rs::Encoding>, line: &Line) -> String {
        let bytes = &line.bytes;
        if let Some(decoder) = encoding {
            decoder.decode(&bytes).0.into_owned()
        } else {
            encoding_rs::SHIFT_JIS.decode(&bytes).0.into_owned()
        }
    }
    pub fn clear(&mut self) {
        self.pages = vec![Page::new()];
    }

    pub fn last_or_new_line(&mut self, font: i32, style: Option<FontStyle>) -> &mut Line {
        let (used, line_font, line_style) = {
            let line = self.last_line();

            (line.bytes.len() > 0, line.font, line.style.clone())
        };
        if used && (line_font != Some(font) || line_style != style) {
            self.new_line();
            let new_line = self.last_line();
            new_line.font = Some(font);
            new_line.style = style;
            new_line
        } else {
            let line = self.last_line();
            if line.font.is_none() {
                line.font = Some(font)
            }
            if line.style.is_none() {
                line.style = style
            }
            line
        }
    }
    pub fn remove_unused(&mut self) {
        if self.last_line().bytes.len() == 0 {
            self.last_paragraph(false).lines.pop();
        }
        if self.last_paragraph(false).lines.len() == 0 && self.last_paragraph(false).table.is_none() {
            self.last_section().paras.pop();
        }
        if self.last_section().paras.len() == 0 {
            self.last_page().sections.pop();
        }
        if self.last_page().sections.len() == 0 {
            self.pages.pop();
        }
    }
    pub fn new_paragraph(&mut self, follow_table: bool) {
        if follow_table {
            let last_para = self.last_paragraph(follow_table);
            if let Some(table) = last_para.table.as_mut() {
                table.last_cell().paras.push(Paragraph::new());
            }
        } else {
            self.last_section().paras.push(Paragraph::new());
        }
    }
    pub fn last_or_new_paragraph(
        &mut self,
        stylesheet: Option<i32>,
        style: Option<ParagraphStyle>,
        in_table: bool,
    ) -> &mut Paragraph {
        let (used, para_style, para_stylesheet, had_table) = {
            let had_table = self.last_paragraph(false).table.is_some();
            let para = self.last_paragraph(in_table);

            (
                para.lines.len() > 1 || para.lines.last().unwrap().bytes.len() > 0,
                para.style.clone(),
                para.stylesheet,
                had_table,
            )
        };
        if had_table != in_table {
            self.last_section().paras.push(Paragraph::new());
            let new_para = self.last_paragraph(false);
            new_para.stylesheet = stylesheet;
            new_para.style = style;
            if in_table {
                new_para.table = Some(Table::new());
            }

            self.last_paragraph(in_table)
        } else if used && (para_style != style || para_stylesheet != stylesheet) {
            {
                if self.last_line().bytes.len() == 0 {
                    self.remove_unused();
                }
            }
            /*{
                println!("{:?}", Text::decode_line(self.encoding, self.last_line()));
            }*/

            //self.last_section().paras.push(Paragraph::new());
            self.new_paragraph(in_table);
            let new_para = self.last_paragraph(in_table);
            new_para.stylesheet = stylesheet;
            new_para.style = style;
            new_para
        } else {
            let para = self.last_paragraph(in_table);
            if para.stylesheet.is_none() {
                para.stylesheet = stylesheet;
            }
            if para.style.is_none() {
                para.style = style;
            }
            para
        }
    }
    pub fn get_row_border(&mut self) -> Option<&mut RowBorder> {
        let table = &mut self.last_paragraph(false).table;
        if let Some(table) = table {
            let last_row = table.last_row();
            if last_row.border.is_some() {
                Some(last_row.border.as_mut().unwrap())
            } else {
                last_row.border = Some(RowBorder {
                    top: None,
                    right: None,
                    bottom: None,
                    left: None,
                    horizontal: None,
                    vertical: None,
                });
                Some(last_row.border.as_mut().unwrap())
            }
        } else {
            None
        }
    }
    pub fn get_cell_border(&mut self) -> Option<&mut CellBorder> {
        let table = &mut self.last_paragraph(false).table;
        if let Some(table) = table {
            let last_row = table.last_row();
            let opts = if last_row.cell_opt_pos == 0 {
                &mut last_row.cells[0].opts
            } else {
                last_row.cell_opts.get_mut(last_row.cell_opt_pos)?
            };

            if opts.border.is_some() {
                Some(opts.border.as_mut().unwrap())
            } else {
                opts.border = Some(CellBorder {
                    top: None,
                    right: None,
                    bottom: None,
                    left: None,
                });
                Some(opts.border.as_mut().unwrap())
            }
        } else {
            None
        }
    }
    pub fn get_border(&mut self, border_select: BorderSelect) -> Option<&mut Border> {
        match border_select {
            BorderSelect::RowTop => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.top = Some(rb.top.clone().unwrap_or_else(|| Border::new()));
                    rb.top.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::RowLeft => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.left = Some(rb.left.clone().unwrap_or_else(|| Border::new()));
                    rb.left.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::RowRight => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.right = Some(rb.right.clone().unwrap_or_else(|| Border::new()));
                    rb.right.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::RowBottom => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.bottom = Some(rb.bottom.clone().unwrap_or_else(|| Border::new()));
                    rb.bottom.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::RowVertical => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.vertical = Some(rb.vertical.clone().unwrap_or_else(|| Border::new()));
                    rb.vertical.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::RowHorizontal => {
                let rb = self.get_row_border();
                if let Some(rb) = rb {
                    rb.horizontal = Some(rb.horizontal.clone().unwrap_or_else(|| Border::new()));
                    rb.horizontal.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::CellTop => {
                let rb = self.get_cell_border();
                if let Some(rb) = rb {
                    rb.top = Some(rb.top.clone().unwrap_or_else(|| Border::new()));
                    rb.top.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::CellLeft => {
                let rb = self.get_cell_border();
                if let Some(rb) = rb {
                    rb.left = Some(rb.left.clone().unwrap_or_else(|| Border::new()));
                    rb.left.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::CellRight => {
                let rb = self.get_cell_border();
                if let Some(rb) = rb {
                    rb.right = Some(rb.right.clone().unwrap_or_else(|| Border::new()));
                    rb.right.as_mut()
                } else {
                    None
                }
            }
            BorderSelect::CellBottom => {
                let rb = self.get_cell_border();
                if let Some(rb) = rb {
                    rb.bottom = Some(rb.bottom.clone().unwrap_or_else(|| Border::new()));
                    rb.bottom.as_mut()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub fn set_border_type(&mut self, border_select: BorderSelect, border_type: BorderType) {
        let border = self.get_border(border_select);
        if let Some(border) = border {
            border.border_type = border_type;
        }
    }
    pub fn set_border_width(&mut self, border_select: BorderSelect, border_width: usize) {
        let border = self.get_border(border_select);
        if let Some(border) = border {
            border.width = border_width;
        }
    }
    pub fn set_cell_right(&mut self, right: usize) {
        let table = &mut self.last_paragraph(false).table;
        if let Some(table) = table {
            let mut last_row = table.last_row();
            if last_row.cell_opt_pos == 0 {
                last_row.cells[0].opts.right = Some(right);
            } else {
                if let Some(opt) = last_row.cell_opts.get_mut(last_row.cell_opt_pos) {
                    opt.right = Some(right);
                }
            }
            last_row.cell_opt_pos = last_row.cell_opt_pos + 1;
            last_row.cell_opts.push(TableCellOption::new());
        }
    }
}
