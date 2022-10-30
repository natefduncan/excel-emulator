use petgraph::{
    graphmap::DiGraphMap, 
    algo::toposort, 
    dot::{Dot, Config}, 
    visit::Dfs 
}; 
use std::{fmt, cmp::Ordering}; 
use crate::{
    workbook::Sheet,
    parser::{
        parse_str, 
        ast::Expr
    }, 
    reference::Reference, 
    errors::Error,
}; 

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct CellId {
    pub sheet: usize, 
    pub row: usize,
    pub column: usize,
    pub num_row: usize, 
    pub num_col: usize, 
    pub dirty: bool 
}

impl PartialOrd for CellId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellId {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.sheet == other.sheet {
            if self.row != other.row {
                self.row.cmp(&other.row)
            } else {
                self.column.cmp(&other.column)
            }
        } else {
            self.sheet.cmp(&other.sheet)
        }
    }
}

impl From<(usize, usize, usize, usize, usize, bool)> for CellId {
    fn from((sheet, row, column, num_row, num_col, dirty) : (usize, usize, usize, usize, usize, bool)) -> CellId {
        CellId { sheet, row, column, num_row, num_col, dirty }
    }
}

impl fmt::Display for CellId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.sheet, self.row, self.column)
    }
}

pub struct DependencyTree {
    tree: DiGraphMap<CellId, u8>, 
    pub offsets: Vec<CellId>
}

/*
Precedent cells — cells that are referred to by a formula in another cell. For example, if cell D10 contains the formula =B5, then cell B5 is a precedent to cell D10.

Dependent cells — these cells contain formulas that refer to other cells. For example, if cell D10 contains the formula =B5, cell D10 is a dependent of cell B5.
*/

impl Default for DependencyTree {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyTree {
    pub fn new() -> DependencyTree {
        DependencyTree { tree: DiGraphMap::new(), offsets: vec![] }
    }

    pub fn add_formula(&mut self, cell: CellId, formula_text: &str, sheets: &Vec<Sheet>) -> Result<(), Error> {
        let mut chars = formula_text.chars();
        chars.next(); // FIXME: Parse can't handle the = in the front of a formula
        let expression: Expr = parse_str(chars.as_str())?;
        self.add_expression(cell, expression, sheets)?; 
        Ok(())
    }

    pub fn add_expression(&mut self, cell: CellId, expression: Expr, sheets: &Vec<Sheet>) -> Result<(), Error> {
        match expression {
            Expr::Reference { sheet, reference } => {
                let sheet_id = match sheet {
                    Some(s) => {
                        sheets.iter().position(|x|  {
                            x.name == s
                        }).unwrap()
                    }, 
                    None => cell.sheet
                }; 
                let sheet: &Sheet = sheets.get(sheet_id).unwrap(); 
                let reference = Reference::from(reference); 
                let (mut start_row, mut start_col, mut num_rows, mut num_cols) = reference.get_dimensions(); 
                start_row = start_row.max(1); 
                start_col = start_col.max(1); 
                num_rows = num_rows.min(sheet.max_rows);
                num_cols = num_cols.min(sheet.max_columns); 
                let pre_cell: CellId; 
                if reference.is_multi_cell() {
                    pre_cell = CellId::from((sheet_id, start_row, start_col, num_rows, num_cols, false)); 
                    if ! self.cell_exists(&pre_cell) {
                        for c in Reference::get_cells_from_dim(start_row, start_col, num_rows, num_cols) {
                            let sub_cell = CellId::from((sheet_id, c.0, c.1, 1, 1, true)); 
                            if sub_cell != pre_cell {
                                self.add_precedent(&sub_cell, &pre_cell); 
                            }
                        }
                    }
                } else {
                    pre_cell = CellId::from((sheet_id, start_row, start_col, num_rows, num_cols, true)); 
                }
                if pre_cell != cell {
                    self.add_precedent(&pre_cell, &cell); 
                }
            },
            Expr::Infix(_, a, b) => {
                self.add_expression(cell, *a, sheets)?; 
                self.add_expression(cell, *b, sheets)?; 
            }, 
            Expr::Prefix(_, a) => {
                self.add_expression(cell, *a, sheets)?; 
            }, 
            Expr::Func { name, args } => {
                if name.as_str() == "OFFSET" {
                    let mut offset_args = args.clone(); 
                    offset_args.remove(0); 
                    self.add_expression(cell, Expr::Array(offset_args), sheets)?; 
                }
                for arg in args.into_iter() {
                    self.add_expression(cell, arg, sheets)?; 
                }
            }, 
            Expr::Array(arr) => {
                for a in arr.into_iter() {
                    self.add_expression(cell, a, sheets)?; 
                }
            }, 
            _ => {}
            
        }
        Ok(())
    }

    pub fn add_cell(&mut self, cell: CellId) {
        self.tree.add_node(cell); 
    }

    pub fn cell_exists(&self, cell: &CellId) -> bool {
        self.tree.contains_node(*cell)
    }

    pub fn add_cell_if_missing(&mut self, cell: &CellId) {
        if self.tree.contains_node(*cell) {
            self.add_cell(*cell); 
        }
    }

    pub fn add_precedent(&mut self, precedent: &CellId, cell: &CellId) {
        self.add_cell_if_missing(precedent);
        self.add_cell_if_missing(cell);
        if !self.tree.contains_edge(*cell, *precedent) {
            self.tree.add_edge(*precedent, *cell, 0); 
        }
   } 

    pub fn is_precedent_of(&self, cell1: &CellId, cell2: &CellId) -> bool {
        self.tree.contains_edge(*cell1, *cell2)
    }

    pub fn is_dependent_of(&self, cell1: &CellId, cell2: &CellId) -> bool {
        self.tree.contains_edge(*cell2, *cell1) 
    } 

    pub fn get_order(&self) -> Vec<CellId> {
        match toposort(&self.tree, None) {
            Ok(order) => {
                order
                // order.into_iter().rev().collect::<Vec<CellId>>()
            }, 
            Err(e) => panic!("{:?}", e) 
        } 
    } 

    pub fn mark_for_recalculation(&mut self, root: &CellId) {
        let mut dfs = Dfs::new(&self.tree, root.clone());
        while let Some(mut node_id) = dfs.next(&self.tree) {
            node_id.dirty = true; 
        }
    }
}

impl fmt::Display for DependencyTree {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", Dot::with_config(&self.tree, &[Config::EdgeNoLabel]))
    }
}

#[cfg(test)]
mod tests {
    use crate::dependency::*; 

    #[test]
    fn test_precedent() {
        let mut tree = DependencyTree::new(); 
        let a = CellId::from((0,0,0,1,1, true)); 
        let b = CellId::from((1,0,0,1,1, true)); 
        let c = CellId::from((2,0,0,1,1, true)); 
        tree.add_precedent(&a, &b); // A must calculate before B 
        tree.add_precedent(&c, &b); // C must calculate before B 
        assert!(tree.is_dependent_of(&b, &a)); 
        assert_eq!(tree.is_dependent_of(&a, &b), false); 
    }

    #[test]
    fn test_order() {
        let mut tree = DependencyTree::new(); 
        let a = CellId::from((0,0,0,1,1, true)); 
        let b = CellId::from((1,0,0,1,1, true)); 
        let c = CellId::from((2,0,0,1,1, true)); 
        tree.add_precedent(&a, &b); // A must calculate before B 
        tree.add_precedent(&b, &c); // B must calculate before C 
        let mut order: Vec<CellId> = tree.get_order(); 
        assert_eq!(order.pop().unwrap(), c);
        assert_eq!(order.pop().unwrap(), b);
        assert_eq!(order.pop().unwrap(), a);
    }
}

