use ndarray::Array2; 
use crate::cell::Cell; 
use crate::range::Range; 

pub struct Sheet {
    pub name: String, 
    pub cells: Array2<Cell>
}

impl Sheet {
    fn get_range(&self, a1: &str) -> Range {
        let mut range = Range::from(a1); 
        range.sheet_name = Some(self.name.clone()); 
        range
     }
}
