
use std::cmp::Ordering; 
use std::fmt; 
use std::hash::Hash;

use crate::evaluate::value::Value;
use crate::parser::ast::Expr; 

// Coordinates for a cell on a sheet,
// starting with 0
#[derive(Clone, Debug, Copy, PartialEq, Eq, Hash)]
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

impl fmt::Display for CellIndex {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{},{}", self.row, self.column)
    }
}

// Individual cell on a sheet
#[derive(Clone, Debug)]
pub struct Cell {
    pub index: CellIndex, 
    pub value: Value, 
    pub function: Option<Expr>
}
