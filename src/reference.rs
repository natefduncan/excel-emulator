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

impl From<String> for Reference {
    fn from(a1 : String) -> Reference {
        if !a1.contains(':') {
            // Single Cell (A1)
            let cell = Cell::from(a1); 
            Reference::from(cell) 
        } else {
            // Range (A1:A1), VRange(A:A), and HRange(1:1)
            let mut cells_split = a1.split(':').map(|x| x.to_owned()).collect::<Vec<String>>(); 
            let c1: String = cells_split.remove(0); 
            let c2: String = cells_split.remove(0); 
            Reference::from((Cell::from(c1), Some(Cell::from(c2))))
        }
    }
}

impl From<&str> for Reference {
    fn from(s : &str) -> Reference {
        Reference::from(s.to_owned())
    }
}

impl fmt::Display for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.is_multi_cell() {
            // Multiple Columns or Rows
            write!(
                f, 
                "{}:{}",
                self.start_cell,
                self.end_cell.as_ref().unwrap()
            )
        } else {
            write!(f, "{}", self.start_cell)
        }
    }
}

impl fmt::Debug for Reference {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl Reference { 
    pub fn get_dimensions(&self) -> (usize, usize, usize, usize) {
        (
            self.row(),
            self.column(),
            self.num_rows(),
            self.num_cols(),
        )
    }

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

    pub fn offset(&mut self, offset: (i32, i32)) {
        if !self.start_cell.row.anchor && !self.start_cell.is_vrange() {
                self.start_cell.row.index = (self.row() as i32 + offset.0) as usize;
        }

        if !self.start_cell.column.anchor && !self.start_cell.is_hrange() {
                self.start_cell.column.index = (self.column() as i32 + offset.1) as usize;
        }

        if self.end_cell.is_some() {
            if !self.end_cell.as_ref().unwrap().row.anchor && !self.end_cell.as_ref().unwrap().is_vrange() {
                    self.end_cell = Some(Cell::from(((self.end_cell.as_ref().unwrap().row.index as i32 + offset.0) as usize, self.end_cell.as_ref().unwrap().column.index)))
            }

            if !self.end_cell.as_ref().unwrap().column.anchor && !self.end_cell.as_ref().unwrap().is_hrange() {
                self.end_cell = Some(Cell::from((self.end_cell.as_ref().unwrap().row.index, (self.end_cell.as_ref().unwrap().column.index as i32 + offset.1) as usize)))
            }
        }
    }
}

