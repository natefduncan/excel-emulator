use std::fmt;
use std::cmp::Ordering; 
use std::hash::{Hasher, Hash}; 

#[derive(Debug, Clone, Copy, Eq)]
pub struct CellIndex {
    pub index : usize,
    pub anchor : bool
}

impl PartialEq for CellIndex {
    fn eq(&self, other: &Self) -> bool {
        self.index == other.index
    }
}

impl Hash for CellIndex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.index.hash(state);
    }
}

impl From<usize> for CellIndex {
    fn from(f: usize) -> CellIndex {
        CellIndex {
            index : f, 
            anchor : false
        }
    }
}

impl From<(usize, bool)> for CellIndex {
    fn from(a : (usize, bool)) -> CellIndex {
        CellIndex {
            index : a.0, 
            anchor : a.1
        }
    }
}
// For a cell to represent an hrange or a vrange,
// utilize a 0 index for the row or column that doens't exist.
// F.ex. vrange will have column CellIndex > 0 with row CellIndex 0
#[derive(Clone, Copy, Eq)]
pub struct Cell {
    pub row : CellIndex, 
    pub column : CellIndex
} 

impl Hash for Cell {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.row.hash(state);
        self.column.hash(state);
    }
}

impl PartialOrd for Cell {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Cell {
    // Top to Bottom, Left to Right
    fn cmp(&self, other: &Self) -> Ordering {
        if self.column.index == other.column.index {
            self.row.index.cmp(&other.row.index)
        } else {
            self.column.index.cmp(&other.column.index)
        }
    }
}

impl PartialEq for Cell {
    fn eq(&self, other: &Self) -> bool {
        (self.row.index == other.row.index) &
        (self.column.index == other.column.index) 
    }
}

impl From<String> for Cell {
    fn from(range : String) -> Cell {
        let alpha = String::from("abcdefghijklmnopqrstuvwxyz");
        // Check if vrange
        if range.chars().filter(|c| c.is_numeric()).count() == 0 {
            let col_anchor : bool = range.starts_with('$');
            let col_str = range
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            let mut col = 0;
            for (_, c) in col_str.to_lowercase().chars().enumerate() {
                let c_i = alpha.chars().position(|r| r == c).unwrap();
                col = col * 26 + c_i + 1;
            }
            Cell {
                row: CellIndex::from((0, false)), 
                column: CellIndex::from((col, col_anchor))
            }
        // Check if hrange
        } else if range.chars().filter(|c| c.is_alphabetic()).count() == 0 {
            let row_anchor : bool = range.starts_with('$');
            let row_num_str = range.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
            let row: usize = row_num_str.parse().unwrap();
            Cell {
                row: CellIndex::from((row, row_anchor)), 
                column: CellIndex::from((0, false))
            }
        } else {
            let col_anchor : bool = range.starts_with('$');
            let row_anchor : bool = match col_anchor {
                true => {
                    range.chars().filter(|c| c == &'$').count() > 1
                }, 
                false => {
                    range.contains('$')
                }
            }; 
            let col_str = range
                .chars()
                .filter(|c| c.is_alphabetic())
                .collect::<String>();
            let row_num_str = range.chars().filter(|c| c.is_ascii_digit()).collect::<String>();
            let row: usize = row_num_str.parse().unwrap();
            let mut col = 0;
            for (_, c) in col_str.to_lowercase().chars().enumerate() {
                let c_i = alpha.chars().position(|r| r == c).unwrap();
                col = col * 26 + c_i + 1;
            }
            Cell {
                row : CellIndex::from((row, row_anchor)),
                column : CellIndex::from((col, col_anchor))
            }
        }
    }
}

impl From<&str> for Cell {
    fn from(s : &str) -> Cell {
        Cell::from(s.to_owned())
    }
}

impl From<(usize, usize)> for Cell {
    fn from((a, b) : (usize, usize)) -> Cell {
        Cell {
            row : CellIndex::from(a), 
            column : CellIndex::from(b)
        }
    
    }
}

impl Cell {
    pub fn as_tuple(self) -> (usize, usize) {
        (self.row.index, self.column.index)
    }

    pub fn is_hrange(self) -> bool {
        self.row.index > 0 && self.column.index == 0
    }

    pub fn is_vrange(self) -> bool {
        self.column.index > 0 && self.row.index == 0
    }

    pub fn column_number_to_letter(col_idx: usize) -> String {
        let alpha = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
        let mut col_name: Vec<char> = vec![];
        let mut n = col_idx;
        while n > 0 {
            let rem: usize = n % 26;
            if rem == 0 {
                col_name.push('Z');
                n = (n / 26) - 1;
            } else {
                col_name.push(alpha.chars().nth(rem - 1).unwrap());
                n /= 26;
            }
        }
        col_name.into_iter().rev().collect::<String>()
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut output = String::new(); 
        if self.column.index != 0 {
            let col_string : String = Cell::column_number_to_letter(self.column.index);
            if self.column.anchor {
                output = format!("${}", col_string);
            } else {
                output = col_string;
            }
        }
        if self.row.index != 0 {
            let row_string : String = self.row.index.to_string();
            if self.row.anchor {
                output = format!("{}${}", output, row_string);
            } else {
                output = format!("{}{}", output, row_string);
            }
        }
        write!(f, "{}", output)
    }
}

impl fmt::Debug for Cell {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    #[test]
    fn parse_from_string() {
        assert_eq!(Cell::from("A1").to_string(), String::from("A1"));
        assert_eq!(Cell::from("B21").to_string(), String::from("B21"));
        assert_eq!(Cell::from("AA1").to_string(), String::from("AA1"));
        assert_eq!(Cell::from("CB100").to_string(), String::from("CB100"));
        assert_eq!(Cell::from("DD2").to_string(), String::from("DD2"));
        assert_eq!(Cell::from("AAA10").to_string(), String::from("AAA10"));
        assert_eq!(Cell::from("GM1").to_string(), String::from("GM1"));
        assert_eq!(Cell::from("ZZ30").to_string(), String::from("ZZ30"));
        assert_eq!(Cell::from("KJ15").to_string(), String::from("KJ15"));
        assert_eq!(Cell::from("A").to_string(), String::from("A"));
        assert_eq!(Cell::from("12").to_string(), String::from("12"));
    }
}
