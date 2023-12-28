use ndarray::Array2; 
use std::sync::{Arc, Mutex};
use crate::cell::Cell; 
use crate::range::{SheetRange, Range}; 

// A page or tab within a spreadsheet
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sheet {
    pub properties: SheetProperties, 
    pub start_row: usize, 
    pub start_column: usize, 
    pub data: Array2<Cell>, 
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SheetProperties {
    pub title: String, 
    pub index: usize, 
    pub sheet_id: usize, 
}

impl Sheet {
    pub fn get_range(&mut self, a1: &str) -> SheetRange {
        SheetRange {
            sheet_ref: Arc::new(Mutex::new(self)), 
            range: Range::from(a1), 
        }
     }
}
