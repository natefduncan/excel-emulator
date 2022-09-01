use quick_xml::events::BytesStart;
use zip::read::{ZipArchive, ZipFile};
use std::fs::File;
use std::fmt; 
use std::io::BufReader; 
use std::collections::HashMap; 
use quick_xml::{
    Reader, 
    events::{
        Event, BytesText, 
        attributes::Attribute
    }, 
};
use anyhow::Result; 
use ndarray::{Array2, Array}; 
use crate::{
    evaluate::Value, 
    utils::adjust_formula, 
    dependency::{CellId, DependencyTree}, 
    utils::excel_to_date, 
    reference::Reference,
    cell::Cell, 
}; 

pub type ZipType = ZipArchive<File>; 

pub struct Book {
    zip: ZipType, 
    sheets: Vec<Sheet>, 
    shared_strings: Vec<SharedString>, 
    styles: Vec<Style>, 
    dependencies: DependencyTree, 
    cells: HashMap<Sheet, Array2<Value>>
}

impl From<String> for Book {
    fn from(s: String) -> Self {
        let zip = Self::zip_from_path(&s); 
        Book { zip, sheets: vec![], shared_strings: vec![], styles: vec![], cells: HashMap::new(), dependencies: DependencyTree::new() }
    }
}

impl From<&str> for Book {
    fn from(s: &str) -> Self {
        Book::from(s.to_string())
    }
}

impl Book {
    pub fn load(&mut self) -> Result<()> {
        self.load_sheet_names()?; 
        self.load_shared_strings()?; 
        self.load_styles()?; 
        self.load_sheets()?; 
        Ok(())
    }

    pub fn load_shared_strings(&mut self) -> Result<()> {
        let mut buf = Vec::new(); 
        if let Ok(f) = self.zip.by_name("xl/sharedStrings.xml") {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            let mut is_shared_string: bool = false; 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name() == b"t" => {
                        is_shared_string = true;
                    }, 
                    Ok(Event::Text(ref e)) => {
                        if is_shared_string {
                            let decoded_text: String = Self::decode_text_event(&reader, e);
                            if !decoded_text.is_empty() {
                                self.shared_strings.push(
                                    SharedString(Box::new(Self::decode_text_event(&reader, e)))
                                )
                            }
                        }
                    }, 
                    Ok(Event::Eof) => break, 
                    _ => {}
                }
                buf.clear(); 
            }
        }
       Ok(())
    }

    pub fn load_styles(&mut self) -> Result<()> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.by_name("xl/styles.xml") {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            let mut is_cell_xfs: bool = false;
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name() == b"cellXfs" || e.name() == b"xf" => {
                        match e.name() {
                            b"cellXfs" => { is_cell_xfs = true; },
                            b"xf" => {
                                if is_cell_xfs {
                                    self.styles.push(Book::decode_style(&reader, e)); 
                                }
                            },
                            _ => {}
                        }
                    }, 
                    Ok(Event::Empty(ref e)) if e.name() == b"xf" => {
                        if is_cell_xfs {
                            self.styles.push(Book::decode_style(&reader, e)); 
                        }
                    }, 
                    Ok(Event::End(ref e)) if e.name() == b"cellXfs" => { is_cell_xfs = false; }, 
                    Ok(Event::Eof) => break, 
                    _ => {}
                }
                buf.clear(); 
            }
        }
        Ok(())
    }

    pub fn load_sheet_names(&mut self) -> Result<()> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.by_name("xl/workbook.xml") {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Empty(ref e)) if e.local_name() == b"sheet" => {
                        for a in e.attributes() {
                            let a = a.unwrap();
                            if let b"name" = a.key {
                                let name = a.unescape_and_decode_value(&reader).unwrap();
                                self.sheets.push(Sheet(name)); 
                            }
                        }
                    }, 
                    Ok(Event::Eof) => break, 
                    _ => {}
                }
                buf.clear(); 
            }
        }
        Ok(())
    }

    pub fn load_sheets(&mut self) -> Result<()> { 
        for sheet_id in 0..self.sheets.len() {
            self.load_sheet(sheet_id)?
        }
        Ok(()) 
    }
    pub fn load_sheet(&mut self, sheet_idx: usize) -> Result<()> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.by_name(&format!("xl/worksheets/sheet{}.xml", sheet_idx + 1)) {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            let mut flags = SheetFlags::new(); 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Empty(ref e)) if e.name() == b"dimension" => {
                        for a in e.attributes() {
                            let a = a.unwrap(); 
                            match a.key {
                                b"ref" => {
                                    let dimension: String = a.unescape_and_decode_value(&reader).unwrap(); 
                                    let (_row, _column, num_rows, num_cols) = Reference::from(dimension).get_dimensions(); 
                                    let sheet: Sheet = self.sheets.get(sheet_idx).unwrap().clone();
                                    self.cells.insert(sheet, Array::from_elem((num_rows, num_cols), Value::Empty)); 
                                }, 
                                _ => {}
                            }
                        }
                    }, 
                    Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) if e.name() == b"c" => {
                        for a in e.attributes() {
                            let a = a.unwrap(); 
                            match a.key {
                                b"r" => {
                                    // Cell reference
                                    flags.current_cell_reference = a.unescape_and_decode_value(&reader).unwrap();
                                }, 
                                b"t" => {
                                    // Cell type
                                    let a_value = a.unescape_and_decode_value(&reader).unwrap();
                                    if a_value == *"s" {
                                        flags.is_string = true; 
                                    }
                                },
                                b"s" => {
                                    // Cell style / date
                                    let cell_style_idx: usize = a.unescape_and_decode_value(&reader).unwrap().parse::<usize>().unwrap(); 
                                    let style: &Style = self.styles.get(cell_style_idx).expect("Could not find style index");
                                    if style.number_format_id >= 14 && style.number_format_id <= 22 && style.apply_number_format {
                                        flags.is_date = true;
                                    }
                                }, 
                                _ => {}
                            }
                        }
                    }, 
                    Ok(Event::Start(ref e)) if e.name() == b"f" => {
                        // Formula flag
                        flags.is_formula = true;
                        for a in e.attributes() {
                            let a = a.unwrap();
                            if a.key == b"ref" {
                                flags.is_shared_formula = true; 
                            }
                        }
                    }, 
                    Ok(Event::Empty(ref e)) if e.name() == b"f" => {
                        // Shared formula
                        for a in e.attributes() {
                            let a = a.unwrap();
                            match a.key {
                                b"si" => {
                                    let formula_index: usize = a.unescape_and_decode_value(&reader).unwrap().parse::<usize>().unwrap(); 
                                    let (start_cell, formula_text): &(Cell, String) = flags.shared_formulas.get(formula_index).unwrap(); 
                                    let base_reference = Reference::from(start_cell.as_tuple()); 
                                    let current_cell = Cell::from(flags.current_cell_reference.clone()); 
                                    let current_reference = Reference::from(current_cell.as_tuple());
                                    let adjusted_formula: Value = Value::Formula(format!("={}", adjust_formula(base_reference, current_reference, formula_text.clone()))); 
                                    let sheet = self.sheets.get(sheet_idx).unwrap(); 
                                    let (row, column): (usize, usize) = current_cell.as_tuple(); 
                                    // println!("{}, {}", flags.current_cell_reference, adjusted_formula); 
                                    self.cells.get_mut(&sheet).unwrap()[[row-1, column-1]] = adjusted_formula.clone(); 
                                    self.dependencies.add_formula(CellId {sheet: sheet_idx, row, column}, &adjusted_formula.to_string(), &self.sheets); 
                                    flags.reset(); 
                                }, 
                                _ => {}
                            }
                        }
                    }, 
                    Ok(Event::Start(ref e)) if e.name() == b"v" => {
                        // Value
                        flags.is_value = true; 
                    }, 
                    Ok(Event::Text(ref e)) => {
                        let cell_text = Book::decode_text_event(&reader, e); 
                        if !cell_text.is_empty() && !flags.current_cell_reference.is_empty() {
                            let cell_text = Book::decode_text_event(&reader, e); 
                            let value: Value; 
                            if flags.is_formula {
                                value = Value::Formula(format!("={}", &cell_text.replace("_xlfn.", "").to_owned())); 
                                if flags.is_shared_formula {
                                    flags.shared_formulas.push(
                                        (Cell::from(flags.current_cell_reference.clone()), cell_text.clone())
                                    )
                                }
                            } else if flags.is_string {
                                let shared_string_idx: usize = cell_text.parse::<usize>().unwrap();
                                let SharedString(s) = self.shared_strings.get(shared_string_idx).unwrap();
                                value = Value::from(*s.clone()); 
                            } else if flags.is_date {
                                value = Value::from(excel_to_date(cell_text.parse::<f64>().unwrap())); 
                            } else if !cell_text.is_empty() {
                                value = match &*cell_text {
                                    "TRUE" => Value::Bool(true), 
                                    "FALSE" => Value::Bool(false), 
                                    _ => {
                                        Value::Num(cell_text.parse::<f32>().expect("Unable to parse to number"))
                                    }
                                }; 
                            } else {
                                value = Value::Empty; 
                            }
                            // println!("{}, {}", flags.current_cell_reference, value); 
                            let sheet = self.sheets.get(sheet_idx).unwrap(); 
                            let cell = Cell::from(flags.current_cell_reference.clone()); 
                            let (row, column): (usize, usize) = cell.as_tuple(); 
                            if value.is_formula() {
                                self.dependencies.add_formula(CellId {sheet: sheet_idx, row, column}, &value.to_string(), &self.sheets); 
                            }
                            self.cells.get_mut(&sheet).unwrap()[[row-1, column-1]] = value; 
                           flags.reset(); 
                        }
                    }, 
                    Ok(Event::Eof) => break, 
                    _ => {} 
                }
            }
        }
        Ok(())
    }

    pub fn zip_from_path(path: &str) -> ZipType {
        let file: File = File::open(path).expect("Unable to find file"); 
        zip::ZipArchive::new(file).expect("Unable to create zip") 
    }

    pub fn decode_text_event(reader: &Reader<BufReader<ZipFile>>, e: &BytesText) -> String {
        e.unescape_and_decode(reader).unwrap()
    }

    pub fn decode_attribute_usize(reader: &Reader<BufReader<ZipFile>>, a: Attribute) -> usize {
        a.unescape_and_decode_value(reader)
        .unwrap()
        .parse::<usize>()
        .unwrap() 
    }

    pub fn decode_style(reader: &Reader<BufReader<ZipFile>>, e: &BytesStart) -> Style {
        let mut number_format_id : usize = 0; 
        let mut apply_number_format: bool = false; 
        for a in e.attributes() {
            let a = a.unwrap(); 
            match a.key {
                b"numFmtId" => {
                    number_format_id = Book::decode_attribute_usize(reader, a); 
                }, 
                b"applyNumberFormat" => {
                    apply_number_format = Book::decode_attribute_usize(reader, a) != 0; 
                }, 
                _ => {}
            }
        }
        Style { number_format_id, apply_number_format }
    }

    pub fn get_sheet_by_name(&self, s: &str) -> &Array2<Value> {
       self.cells.get(&Sheet(s.to_owned())).unwrap()
    }

    pub fn get_sheet_by_idx(&self, idx: usize) -> &Array2<Value> {
        self.cells.get(&self.sheets[idx]).unwrap()
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Sheet(pub String); 
impl From<&str> for Sheet {
    fn from(s: &str) -> Sheet {
        Sheet(s.to_string())
    }
}

impl fmt::Display for Sheet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let Sheet(s) = self;
        write!(f, "'{}'!", s)
    }
}

impl Sheet {
    pub fn name(&self) -> String {
        let Sheet(s) = self;
        s.clone()
    }
}

#[derive(Debug)]
pub struct SharedString(Box<String>); 
pub struct Style {
    pub number_format_id: usize, 
    pub apply_number_format: bool 
}

impl Default for Style {
    fn default() -> Style {
        Style { number_format_id: 0, apply_number_format: false }
    }
}

impl Style {
    pub fn new() -> Style {
        Default::default()
    }

}

#[derive(Debug)]
struct SheetFlags {
    is_shared_formula: bool, 
    is_date: bool, 
    is_formula: bool, 
    is_string: bool, 
    is_value: bool, 
    current_cell_reference: String, 
    shared_formulas: Vec<(Cell, String)>, // Start Cell, Formula
}

impl SheetFlags {
    fn new() -> SheetFlags {
        SheetFlags {
            is_shared_formula: false, 
            is_date: false, 
            is_formula: false, 
            is_string: false, 
            is_value: false, 
            current_cell_reference: String::new(), 
            shared_formulas: vec![]
        }
    }
    
    fn reset(&mut self) {
        self.is_shared_formula = false; 
        self.is_date = false; 
        self.is_formula = false;
        self.is_string = false; 
        self.is_value = false; 
        self.current_cell_reference = String::new(); 
    }
}

#[cfg(test)]
mod tests {
    use crate::workbook::Book;
    use crate::evaluate::Value;

    #[test]
    fn test_sheet_names() {
        let mut book = Book::from("assets/data_types.xlsx"); 
        book.load().expect("Could not load workbook"); 
        assert_eq!(&book.sheets[0].name(), "test 1");
        assert_eq!(&book.sheets[1].name(), "test 2");
        assert_eq!(&book.sheets[2].name(), "test 3");
        println!("{}", book.dependencies); 
        println!("{:?}", book.dependencies.get_order()); 
        assert_eq!(&book.sheets[2].name(), "test 4");
    }

    #[test]
    fn test_cells() {
        let mut book = Book::from("assets/data_types.xlsx"); 
        book.load().expect("Could not load workbook"); 
        assert_eq!(book.get_sheet_by_name("test 1")[[0, 0]], Value::from("Text")); 
        assert_eq!(book.get_sheet_by_name("test 1")[[1, 0]], Value::from("a")); 
        assert_eq!(book.get_sheet_by_name("test 1")[[2, 0]], Value::from("b")); 
        assert_eq!(book.get_sheet_by_name("test 1")[[3, 0]], Value::from("c")); 
        assert_eq!(book.get_sheet_by_name("test 1")[[1, 4]], Value::Formula(String::from("=B2+1"))); 
        assert_eq!(book.get_sheet_by_name("test 1")[[2, 4]], Value::Formula(String::from("=B3+1"))); 
        assert_eq!(book.get_sheet_by_name("test 1")[[3, 4]], Value::Formula(String::from("=(B4+1)")));
    }
}

