
use std::cmp::Ordering; 
use std::hash::Hash;

use crate::evaluate::value::Value;
use crate::parser::ast::Expr; 

// Coordinates for a cell on a sheet,
// starting with 0
#[derive(Clone, Debug, Copy, Eq, Hash)]
pub struct CellIndex {
    pub row : usize, 
    pub column : usize 
} 

impl CellIndex {
    pub fn is_hrange(self) -> bool {
        self.row > 0 && self.column == 0
    }

    pub fn is_vrange(self) -> bool {
        self.column > 0 && self.row == 0
    }

    pub fn as_tuple(self) -> (usize, usize) {
        (self.row, self.column)
    }
}

impl PartialEq for CellIndex {
    fn eq(&self, other: &Self) -> bool {
        (self.row == other.row) &
        (self.column == other.column) 
    }
}

impl PartialOrd for CellIndex {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CellIndex {
    // Top to Bottom, Left to Right
    fn cmp(&self, other: &Self) -> Ordering {
        if self.column == other.column {
            self.row.cmp(&other.row)
        } else {
            self.column.cmp(&other.column)
        }
    }
}

// (row, column)
impl From<(usize, usize)> for CellIndex {
    fn from(coords: (usize, usize)) -> CellIndex {
        let (row, column) = coords; 
        CellIndex { row, column }
    }
}

// Individual cell on a sheet
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cell {
    index: CellIndex, 
    value: Value, 
    function: Option<Expr>
}
