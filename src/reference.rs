use std::fmt; 
use std::cmp::Ordering; 
use std::hash::{Hasher, Hash}; 

use crate::{
    cell::Cell,
    sheet::SheetName, 
}; 

#[derive(Clone, Copy, Eq)]
pub struct Reference {
    pub sheet_name: String, 
    pub range: Range,
}

pub struct Reference {
    pub start_cell : Cell, 
    pub end_cell : Option<Cell>
}

impl PartialOrd for Reference {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Hash for Reference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.start_cell.hash(state); 
        self.end_cell.hash(state); 
    }
}

impl Ord for Reference {
    // Top to Bottom, Left to Right
    fn cmp(&self, other: &Self) -> Ordering {
        self.start_cell.cmp(&other.start_cell)
    }
}

impl PartialEq for Reference {
    fn eq(&self, other: &Self) -> bool {
        (self.start_cell == other.start_cell)
        & (self.end_cell == other.end_cell) 
    }
}

impl From<Cell> for Reference {
    fn from(a : Cell) -> Reference {
        Reference {
            start_cell : a, 
            end_cell : None
        }
    }
}

impl From<(Cell, Option<Cell>)> for Reference {
    fn from((a, b) : (Cell, Option<Cell>)) -> Reference {
        Reference {
            start_cell : a, 
            end_cell : b
        }
    }
}

impl From<(usize, usize)> for Reference {
    fn from((a, b) : (usize, usize)) -> Reference {
        Reference {
            start_cell : Cell::from((a, b)), 
            end_cell : None
        }
    }
}

impl From<(usize, usize, usize, usize)> for Reference {
    fn from((a, b, c, d) : (usize, usize, usize, usize)) -> Reference {
        Reference {
            start_cell : Cell::from((a, b)),
            end_cell : Some(Cell::from((c, d)))
        }
    }
}

impl fmt::Debug for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Reference { 
    pub fn get_cells_from_dim(start_row: usize, start_column: usize, num_rows: usize, num_cols: usize) -> Vec<(usize, usize)> {
        let mut output: Vec<(usize, usize)> = vec![]; 
        for row in start_row..(start_row + num_rows) {
            for column in start_column..(start_column + num_cols) {
                output.push((row, column)); 
            }
        }
        output
    }

    pub fn get_cells(&self) -> Vec<(usize, usize)> {
        let (start_row, start_column, num_rows, num_cols) = self.get_dimensions();
        Self::get_cells_from_dim(start_row, start_column, num_rows, num_cols)
    }
}

