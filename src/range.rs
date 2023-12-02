// Range is only used in formulas and includes anchors ($). 
// It can be single cell (A1) or multi cell (A1:B2)
use std::fmt;

use crate::{cell::CellIndex, errors::Error, 
    utils}; 

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AnchorType {
    Row, 
    Column,
    Cell, 
    None, 
}

#[derive(Debug, Clone, Eq)]
pub struct Range {
    pub start_cell: CellIndex, 
    pub start_anchor: AnchorType, 
    pub end_cell: Option<CellIndex>,
    pub end_anchor: Option<AnchorType>, 
}

impl PartialEq for Range {
    fn eq(&self, other: &Self) -> bool {
        (self.start_cell == other.start_cell) &
            (self.start_anchor == other.start_anchor) &
            (self.end_cell == other.end_cell) &
            (self.end_anchor == other.end_anchor)
    }
}

impl From<String> for Range {
    fn from(a1 : String) -> Range {
        if !a1.contains(':') {
            // Single Cell (A1)
            let Ok((start_cell, start_anchor)) = parse_range_part(a1) else { todo!() }; 
            Range {
                start_cell, 
                start_anchor,
                end_cell: None,
                end_anchor: None
            }
        } else {
            // Range (A1:A1), VRange(A:A), and HRange(1:1)
            let mut cells_split = a1.split(':').map(|x| x.to_owned()).collect::<Vec<String>>(); 
            let c1: String = cells_split.remove(0); 
            let c2: String = cells_split.remove(0); 
            let Ok((start_cell, start_anchor)) = parse_range_part(c1) else { todo!() }; 
            let Ok((end_cell, end_anchor)) = parse_range_part(c2) else { todo!() }; 
            Range { start_cell, start_anchor, end_cell: Some(end_cell), end_anchor: Some(end_anchor) }
        }
    }
}

impl From<&str> for Range {
    fn from(a1: &str) -> Range {
        Range::from(a1.to_string())
    }
}

// A range part is text for a single cell
// e.g., A1 or B12
// Full range might also include end cell
// e.g., A1:B12 with colon in-between
pub fn parse_range_part(range_part: String) -> Result<(CellIndex, AnchorType), Error> {
    let alpha = String::from("abcdefghijklmnopqrstuvwxyz");
    // Check if vrange (row will be zero)
    if range_part.chars().filter(|c| c.is_numeric()).count() == 0 {
        let anchor_type = if range_part.starts_with('$') {
            AnchorType::Column
        } else {
            AnchorType::None
        }; 
        let col_str = range_part
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>();
        let mut col = 0;
        for (_, c) in col_str.to_lowercase().chars().enumerate() {
            let c_i = alpha.chars().position(|r| r == c).unwrap();
            col = col * 26 + c_i + 1;
        }
        
        Ok((CellIndex::from((0, col)), anchor_type))
    // Check if hrange (column will be zero)
    } else if range_part.chars().filter(|c| c.is_alphabetic()).count() == 0 {
        let anchor_type = if range_part.starts_with('$') {
            AnchorType::Row
        } else {
            AnchorType::None
        }; 
        let row_num_str = range_part.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        let row: usize = row_num_str.parse().unwrap();
        Ok((CellIndex::from((row, 0)), anchor_type))
    } else {
        let col_anchor : bool = range_part.starts_with('$');
        let row_anchor : bool = match col_anchor {
            true => {
                range_part.chars().filter(|c| c == &'$').count() > 1
            }, 
            false => {
                range_part.contains('$')
            }
        }; 
        let anchor_type = if col_anchor & row_anchor {
            AnchorType::Cell
        } else if col_anchor {
            AnchorType::Column
        } else if row_anchor {
            AnchorType::Row
        } else {
            AnchorType::None
        }; 
        let col_str = range_part 
            .chars()
            .filter(|c| c.is_alphabetic())
            .collect::<String>();
        let row_num_str = range_part.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
        let row: usize = row_num_str.parse().unwrap();
        let mut col = 0;
        for (_, c) in col_str.to_lowercase().chars().enumerate() {
            let c_i = alpha.chars().position(|r| r == c).unwrap();
            col = col * 26 + c_i + 1;
        }
        Ok((CellIndex::from((row, col)), anchor_type))
    }
}

impl Range {
    pub fn row(&self) -> usize{
        self.start_cell.row
    }

    pub fn column(&self) -> usize {
        self.start_cell.column
    }

    pub fn is_single_cell(&self) -> bool {
        self.end_cell.is_none() && !self.start_cell.is_hrange() && !self.start_cell.is_vrange()
    }

    pub fn is_multi_cell(&self) -> bool {
        self.end_cell.is_some() || self.start_cell.is_hrange() || self.start_cell.is_vrange()
    }

    pub fn is_hrange(&self) -> bool {
        self.start_cell.is_hrange()
    }

    pub fn is_vrange(&self) -> bool {
        self.start_cell.is_vrange()
    }

    pub fn num_rows(&self) -> usize {
        if self.is_multi_cell() {
            if self.start_cell.is_vrange() {
                usize::MAX
            } else {
                self.end_cell.as_ref().unwrap().row - self.start_cell.row + 1 
            }
        } else {
            1
        }
    }

    pub fn num_cols(&self) -> usize {
        if self.is_multi_cell() {
            if self.start_cell.is_hrange() {
                usize::MAX
            } else {
                self.end_cell.as_ref().unwrap().column - self.start_cell.column + 1
            }
        } else {
            1
        }
    }

    pub fn get_dimensions(&self) -> (usize, usize, usize, usize) {
        (
            self.row(),
            self.column(),
            self.num_rows(),
            self.num_cols(),
        )
    }
}

fn format_range_part(cell_index: &CellIndex, anchor_type: &AnchorType) -> String {
    let output = String::new(); 
    if (cell_index.row != 0) & (cell_index.column != 0) {
        let col_string : String = utils::column_number_to_letter(cell_index.column);
        match anchor_type {
            AnchorType::Row => {
                format!("{}${}", col_string, cell_index.row)
            },
            AnchorType::Column => {
                format!("${}{}", col_string, cell_index.row)
            }, 
            AnchorType::Cell => {
                format!("${}${}", col_string, cell_index.row)
            }, 
            AnchorType::None => {
                format!("{}{}", col_string, cell_index.row)
            }
        }
    } else if cell_index.column != 0 {
        // VRange
        let col_string : String = utils::column_number_to_letter(cell_index.column);
        if anchor_type == &AnchorType::Column {
            format!("${}", col_string)
        } else {
            col_string
        }
    } else {
        // HRange
        let row_string : String = cell_index.row.to_string();
        if anchor_type == &AnchorType::Row {
            format!("{}${}", output, row_string)
        } else {
            format!("{}{}", output, row_string)
        }
    }
}

impl fmt::Display for Range {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let start_format = format_range_part(&self.start_cell, &self.start_anchor); 
        if let Some(end_cell) = self.end_cell {
            if let Some(end_anchor) = &self.end_anchor {
                let end_format = format_range_part(&end_cell, end_anchor); 
                write!(f, "{}{}", start_format, end_format)
            } else {
                panic!("End anchor set when no end cell"); 
            }
        } else {
            write!(f, "{}", start_format)
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::Error;
    use crate::range::Range; 

    #[test]
    fn parse_from_string() -> Result<(), Error> {
        assert_eq!(Range::from("A1").to_string(), String::from("A1"));
        assert_eq!(Range::from("B21").to_string(), String::from("B21"));
        assert_eq!(Range::from("AA1").to_string(), String::from("AA1"));
        assert_eq!(Range::from("CB100").to_string(), String::from("CB100"));
        assert_eq!(Range::from("DD2").to_string(), String::from("DD2"));
        assert_eq!(Range::from("AAA10").to_string(), String::from("AAA10"));
        assert_eq!(Range::from("GM1").to_string(), String::from("GM1"));
        assert_eq!(Range::from("ZZ30").to_string(), String::from("ZZ30"));
        assert_eq!(Range::from("KJ15").to_string(), String::from("KJ15"));
        assert_eq!(Range::from("A").to_string(), String::from("A"));
        assert_eq!(Range::from("12").to_string(), String::from("12"));
        Ok(())
    }
}
