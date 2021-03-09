pub use super::*;

#[derive(Clone, Debug)]
pub struct TableCellOption {
    pub border: Option<CellBorder>,
    pub right: Option<Twips>,
}
impl TableCellOption {
    pub fn new() -> Self {
        Self {
            border: None,
            right: None,
        }
    }
}
#[derive(Clone, Debug)]
pub struct TableCell {
    pub paras: Vec<Paragraph>,
    pub opts: TableCellOption,
}
impl TableCell {
    pub fn new() -> TableCell {
        TableCell {
            paras: vec![Paragraph::new()],
            opts: TableCellOption::new(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.paras.len() == 1 && self.paras[0].is_empty()
    }
}

#[derive(Clone, Debug)]
pub struct TableRow {
    pub cells: Vec<TableCell>,
    pub border: Option<RowBorder>,
    pub cell_opt_pos: usize,
    pub cell_opts: Vec<TableCellOption>,
}
impl TableRow {
    pub fn new() -> TableRow {
        TableRow {
            cells: vec![TableCell::new()],
            border: None,
            cell_opt_pos: 0,
            cell_opts: vec![TableCellOption::new()],
        }
    }
    pub fn add_cell(&mut self) {
        let last_opts = self.cells.last().map(|last| last.opts.clone());
        let pos = self.cells.len();
        let mut new_cell = TableCell::new();
        if let Some(opts) = self.cell_opts.get(pos) {
            new_cell.opts = opts.clone();
        } else if let Some(opts) = last_opts {
            let mut new_opt = TableCellOption::new();
            new_opt.border = opts.border;
            new_cell.opts = new_opt;
        }
        self.cells.push(new_cell);
    }
    pub fn is_empty(&self) -> bool {
        self.cells.len() == 1 && self.cells[0].is_empty()
    }
}
#[derive(Clone, Debug)]
pub struct Table {
    pub rows: Vec<TableRow>,
}
impl Table {
    pub fn new() -> Table {
        Table {
            rows: vec![TableRow::new()],
        }
    }
    pub fn add_row(&mut self) {
        self.rows.push(TableRow::new())
    }
    pub fn last_row(&mut self) -> &mut TableRow {
        self.rows.last_mut().expect("must exist row")
    }
    pub fn last_cell(&mut self) -> &mut TableCell {
        self.rows
            .last_mut()
            .expect("must exist row")
            .cells
            .last_mut()
            .expect("must exist cell")
    }
    pub fn add_cell(&mut self) {
        let row = self.last_row();
        row.add_cell();
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Twips(usize);
impl From<usize> for Twips {
    fn from(n: usize) -> Twips {
        Self { 0: n }
    }
}
impl Twips {
    pub fn into_px(self) -> usize {
        self.0 / 15
    }
}
