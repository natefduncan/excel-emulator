use ndarray::Array2; 
use crate::cell::Cell; 
use crate::range::Range; 

// A page or tab within a spreadsheet
pub struct Sheet {
    pub properties: SheetProperties, 
    pub start_row: usize, 
    pub start_column: usize, 
    pub data: Array2<Cell>, 
}

pub struct SheetProperties {
    pub title: String, 
    pub index: usize, 
    pub sheet_id: usize, 
}

impl Sheet {
    pub fn get_range(&self, a1: &str) -> Range {
        let mut range = Range::from(a1); 
        range.sheet_name = Some(self.properties.title.clone()); 
        range
     }
}
