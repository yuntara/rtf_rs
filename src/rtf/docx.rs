use std::io::SeekFrom;

use super::*;
use std::collections::HashMap;
use std::collections::VecDeque;

pub trait Docx {
    fn into_docx(self) -> Result<Vec<u8>, Errors>;
    fn into_docx_base64(self) -> Result<String, Errors>;
}
impl Docx for Rtf {
    fn into_docx_base64(self) -> Result<String, Errors> {
        let docx = self.into_docx()?;
        Ok(base64::encode(docx))
    }
    fn into_docx(self) -> Result<Vec<u8>, Errors> {
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
        impl std::convert::Into<docx_rs::VAlignType> for CellVerticalAlignment {
            fn into(self) -> docx_rs::VAlignType {
                match self {
                    Self::Top => VAlignType::Top,
                    Self::Center => VAlignType::Center,
                    Self::Bottom => VAlignType::Bottom,
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
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::Top);
                    b.border_type = top.border_type.into();
                    b.size = top.width;
                    borders = borders.set(b);
                }
                if let Some(left) = self.left {
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::Left);
                    b.border_type = left.border_type.into();
                    b.size = left.width;
                    borders = borders.set(b);
                }
                if let Some(right) = self.right {
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::Right);
                    b.border_type = right.border_type.into();
                    b.size = right.width;

                    borders = borders.set(b);
                }
                if let Some(bottom) = self.bottom {
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::Bottom);
                    b.border_type = bottom.border_type.into();
                    b.size = bottom.width;
                    borders = borders.set(b);
                }
                if let Some(vertical) = self.vertical {
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::InsideV);
                    b.border_type = vertical.border_type.into();
                    b.size = vertical.width;
                    borders = borders.set(b);
                }
                if let Some(horizontal) = self.horizontal {
                    let mut b = TableBorder::new(docx_rs::TableBorderPosition::InsideH);
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
                    let mut b = TableCellBorder::new(docx_rs::TableCellBorderPosition::Top);
                    b.border_type = top.border_type.into();
                    b.size = top.width;
                    borders = borders.set(b);
                }
                if let Some(left) = self.left {
                    let mut b = TableCellBorder::new(docx_rs::TableCellBorderPosition::Left);
                    b.border_type = left.border_type.into();
                    b.size = left.width;
                    borders = borders.set(b);
                }
                if let Some(right) = self.right {
                    let mut b = TableCellBorder::new(docx_rs::TableCellBorderPosition::Right);
                    b.border_type = right.border_type.into();
                    b.size = right.width;

                    borders = borders.set(b);
                }
                if let Some(bottom) = self.bottom {
                    let mut b = TableCellBorder::new(docx_rs::TableCellBorderPosition::Bottom);
                    b.border_type = bottom.border_type.into();
                    b.size = bottom.width;
                    borders = borders.set(b);
                }
                borders
            }
        }
        fn make_runs(
            line: &Line,
            font_table: &HashMap<i32, font::Font>,
            encoding: Option<&'static encoding_rs::Encoding>,
            stylesheet_font_style: &FontStyle,
            color_table: &[color::Color],
            default_font: Option<i32>,
        ) -> std::collections::VecDeque<Run> {
            let mut run = Run::new();

            let text = if let Some(font) = line.font.or(default_font) {
                if let Some(font) = font_table.get(&font) {
                    let text = match font.charset {
                        Some(Charset::ShiftJIS) if encoding != Some(encoding_rs::UTF_16LE) => {
                            crate::rtf::Text::decode_line(Some(encoding_rs::SHIFT_JIS), &line)
                        }
                        _ => crate::rtf::Text::decode_line(encoding, &line),
                    };
                    let run_font = RunFonts::new().east_asia(font.font_name.clone());

                    run = run.fonts(run_font);
                    // println!("{} {:?} {:?}", text, encoding, font.charset);
                    text
                } else {
                    crate::rtf::Text::decode_line(encoding, &line)
                }
            } else {
                crate::rtf::Text::decode_line(encoding, &line)
            };

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
                if style.foreground_color > 0 {
                    if let Some(color) = color_table.get(style.foreground_color - 1) {
                        run = run.color(color)
                    }
                }
                if style.background_color > 0 {
                    if let Some(color) = color_table.get(style.background_color - 1) {
                        run = run.highlight(color)
                    }
                }
            }
            let texts = text.split('\n');
            let mut runs = VecDeque::new();
            for text in texts {
                runs.push_back(run.clone().add_text(text));
            }
            runs
        }
        use docx_rs::*;
        use std::io::{Cursor, Read, Seek};
        let mut docx = docx_rs::Docx::new();
        let mut cursor = Cursor::new(Vec::new());

        let (text, font_table, stylesheets, color_table, default_font_number) = self.get_text();
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
                            let mut grid: Vec<usize> = vec![];
                            let mut make_grid = false;

                            for rtf_row in table.rows {
                                if rtf_row.border.is_some() {
                                    border = rtf_row.border.clone();
                                }
                                let mut left = Some(0);
                                if rtf_row.is_empty() {
                                    continue;
                                }
                                let mut cells: Vec<docx_rs::TableCell> = vec![];
                                let cell_len = rtf_row.cells.len();

                                for (cell_index, rtf_cell) in rtf_row.cells.into_iter().enumerate()
                                {
                                    if cell_index == cell_len - 1 && rtf_cell.is_empty() {
                                        continue;
                                    }

                                    let width = if let Some(left) = left {
                                        rtf_cell.opts.right.map(|r| r.into_px() - left)
                                    } else {
                                        None
                                    };
                                    if let Some(width) = width {
                                        if left.is_some() {
                                            left = Some(left.unwrap() + width);
                                        }
                                    }
                                    let mut cell = docx_rs::TableCell::new();
                                    if let Some(border) = rtf_cell.opts.border {
                                        cell = cell.set_borders(border.into());
                                    } else {
                                    }

                                    if rtf_cell.opts.vert_merge_root {
                                        cell = cell.vertical_merge(VMergeType::Restart);
                                    } else if rtf_cell.opts.vert_merged_cell {
                                        cell = cell.vertical_merge(VMergeType::Continue);
                                    } else {
                                    }
                                    cell = cell.vertical_align(rtf_cell.opts.vert_align.into());

                                    for para in rtf_cell.paras {
                                        let para_style =
                                            para.style.as_ref().unwrap_or(&stylesheet_para);
                                        let align = para_style
                                            .align
                                            .as_ref()
                                            .or_else(|| stylesheet_para.align.as_ref());

                                        let first_indent = para_style
                                            .first_indent
                                            .as_ref()
                                            .or_else(|| stylesheet_para.first_indent.as_ref());
                                        let special_indent = first_indent.map(|indent| {
                                            SpecialIndentType::FirstLine(indent.clone())
                                        });

                                        let make_paragrah = || {
                                            let mut p = Paragraph::new();

                                            if let Some(align) = align {
                                                p = p.align(align.clone().into());
                                            }
                                            p = p.indent(
                                                para_style
                                                    .left_indent
                                                    .or(stylesheet_para.left_indent),
                                                special_indent,
                                                para_style
                                                    .right_indent
                                                    .or(stylesheet_para.right_indent),
                                                None,
                                            );
                                            p
                                        };
                                        let mut runs: VecDeque<Run> = VecDeque::new();
                                        let process_run =
                                            |cell: TableCell, runs: &mut VecDeque<Run>| {
                                                if !runs.is_empty() {
                                                    let mut p = make_paragrah();
                                                    while let Some(run) = runs.pop_front() {
                                                        p = p.add_run(run.clone());
                                                    }
                                                    cell.add_paragraph(p)
                                                } else {
                                                    cell
                                                }
                                            };

                                        for line in para.lines {
                                            let mut splitted = make_runs(
                                                &line,
                                                &font_table,
                                                line.encoding.or(text.encoding),
                                                &stylesheet_font_style,
                                                &color_table,
                                                default_font_number,
                                            );

                                            runs.push_back(splitted.pop_front().unwrap());

                                            for run in splitted {
                                                cell = process_run(cell, &mut runs);
                                                runs.push_back(run);
                                            }
                                        }
                                        cell = process_run(cell, &mut runs);
                                    }
                                    if let Some(width) = width {
                                        cell = cell.width(width, WidthType::DXA);
                                        if make_grid {
                                            grid.push(Twips::from_px(width).into());
                                        }
                                    } else {
                                        left = None;
                                        grid = vec![];
                                        make_grid = false;
                                    }

                                    cells.push(cell);
                                }
                                let row = docx_rs::TableRow::new(cells);

                                rows.push(row);
                                make_grid = false;
                            }

                            let mut table = docx_rs::Table::new(rows);
                            if let Some(border) = border {
                                table = table.set_borders(border.into());
                            }
                            if grid.len() > 0 {
                                table = table.set_grid(grid);
                            }
                            docx = docx.add_table(table);
                        } else {
                            let make_paragrah = || {
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
                                p
                            };
                            let mut runs: VecDeque<Run> = VecDeque::new();
                            let process_run = |docx: Docx, runs: &mut VecDeque<Run>| {
                                if runs.len() > 0 {
                                    let mut p = make_paragrah();
                                    loop {
                                        if let Some(run) = runs.pop_front() {
                                            p = p.add_run(run.clone());
                                        } else {
                                            break;
                                        }
                                    }
                                    docx.add_paragraph(p)
                                } else {
                                    docx
                                }
                            };

                            for line in para.lines {
                                let mut splitted = make_runs(
                                    &line,
                                    &font_table,
                                    line.encoding.or(text.encoding),
                                    &stylesheet_font_style,
                                    &color_table,
                                    default_font_number,
                                );
                                runs.push_back(splitted.pop_front().unwrap());

                                for run in splitted {
                                    docx = process_run(docx, &mut runs);
                                    runs.push_back(run);
                                }
                            }
                            docx = process_run(docx, &mut runs);
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
