use crate::sheet::Sheet; 

// Collection of sheets
pub struct Spreadsheet {
    pub properties: SpreadsheetProperties, 
    pub sheets: Vec<Sheet>, 
}

pub struct SpreadsheetProperties {
    pub title: String, 
}

impl Spreadsheet {
    pub fn add_sheet(&mut self, sheet: Sheet) {
        self.sheets.push(sheet); 
    }
}
