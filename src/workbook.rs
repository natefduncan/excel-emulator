use quick_xml::events::BytesStart;
use zip::read::{ZipArchive, ZipFile};
use indicatif::ProgressBar; 
use std::fs::File;
use std::fmt; 
use std::io::BufReader; 
use quick_xml::{
    Reader, 
    events::{
        Event, BytesText, 
        attributes::Attribute
    }, 
};
use ndarray::{Array2, Array, s, Axis, ArrayView}; 
use crate::{
    evaluate::{
        value::Value, 
        evaluate_expr_with_context, 
        ensure_non_range
    }, 
    utils::adjust_formula, 
    dependency::{CellId, DependencyTree}, 
    utils::excel_to_date, 
    reference::Reference,
    parser::{
        parse_str, 
        ast::Expr
    }, 
    cell::Cell, 
    errors::Error
}; 

pub type ZipType = ZipArchive<File>; 

pub struct Book {
    zip: Option<ZipType>, 
    pub sheets: Vec<Sheet>, 
    shared_strings: Vec<SharedString>, 
    styles: Vec<Style>, 
    pub current_sheet: usize, 
    pub dependencies: DependencyTree, 
    // pub cells: HashMap<Sheet, Array2<Value>>
}

impl From<String> for Book {
    fn from(s: String) -> Self {
        let zip = Self::zip_from_path(&s); 
        Book { zip: Some(zip), sheets: vec![], shared_strings: vec![], styles: vec![], current_sheet: 0, dependencies: DependencyTree::new() }
    }
}

impl From<&str> for Book {
    fn from(s: &str) -> Self {
        Book::from(s.to_string())
    }
}

impl Default for Book {
    fn default() -> Self {
        Self::new()
    }
}

impl Book {
    pub fn new() -> Book {
        Book { zip: None, sheets: vec![], shared_strings: vec![], styles: vec![], current_sheet: 0, dependencies: DependencyTree::new() }
    }

    pub fn load(&mut self, progress: bool) -> Result<(), Error> {
        self.load_sheet_names()?; 
        self.load_shared_strings()?; 
        self.load_styles()?; 
        self.load_sheets_dimensions()?; 
        self.load_sheets(progress)?; 
        Ok(())
    }

    pub fn load_shared_strings(&mut self) -> Result<(), Error> {
        let mut buf = Vec::new(); 
        if let Ok(f) = self.zip.as_mut().unwrap().by_name("xl/sharedStrings.xml") {
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
                                    SharedString(Self::decode_text_event(&reader, e))
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

    pub fn load_styles(&mut self) -> Result<(), Error> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.as_mut().unwrap().by_name("xl/styles.xml") {
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

    pub fn load_sheet_names(&mut self) -> Result<(), Error> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.as_mut().unwrap().by_name("xl/workbook.xml") {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Empty(ref e)) if e.local_name() == b"sheet" => {
                        for a in e.attributes() {
                            let a = a.unwrap();
                            if let b"name" = a.key {
                                let name = a.unescape_and_decode_value(&reader).unwrap();
                                self.sheets.push(Sheet::from(name)); 
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

    pub fn load_sheets(&mut self, progress: bool) -> Result<(), Error> { 
        for sheet_id in 0..self.sheets.len() {
            self.load_sheet(sheet_id, progress)?; 
        }
        Ok(()) 
    }

    pub fn load_sheets_dimensions(&mut self) -> Result<(), Error> {
        for sheet_id in 0..self.sheets.len() {
            self.load_sheet_dimensions(sheet_id)?; 
        }
        Ok(()) 
    }

    pub fn load_sheet_dimensions(&mut self, sheet_idx: usize) -> Result<(), Error> {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.as_mut().unwrap().by_name(&format!("xl/worksheets/sheet{}.xml", sheet_idx + 1)) {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Empty(ref e)) if e.name() == b"dimension" => {
                        for a in e.attributes() {
                            let a = a.unwrap(); 
                            if let b"ref" = a.key {
                                let dimension: String = a.unescape_and_decode_value(&reader).unwrap(); 
                                let (row, column, num_rows, num_cols) = Reference::from(dimension.clone()).get_dimensions(); 
                                let sheet: &mut Sheet = self.sheets.get_mut(sheet_idx).unwrap();
                                sheet.values = Array::from_elem((num_rows + row, num_cols + column), SheetValue::new()); 
                                sheet.max_rows = num_rows + row; 
                                sheet.max_columns = num_cols + column; 
                            }
                        }
                        break
                    }, 
                    _ => {}
                }
            }
        } 
        Ok(())
    }

    pub fn load_sheet(&mut self, sheet_idx: usize, progress: bool) -> Result<(), Error> {
        let mut buf = Vec::new();
        let max_rows = self.get_sheet_by_idx(sheet_idx).max_rows.clone(); 
        let max_columns = self.get_sheet_by_idx(sheet_idx).max_columns.clone(); 
        let pb = match progress {
            true => ProgressBar::new((max_rows * max_columns) as u64), 
            false => ProgressBar::hidden()
        }; 
        if let Ok(f) = self.zip.as_mut().unwrap().by_name(&format!("xl/worksheets/sheet{}.xml", sheet_idx + 1)) {
            let mut reader: Reader<BufReader<ZipFile>> = Reader::<BufReader<ZipFile>>::from_reader(BufReader::new(f)); 
            let mut flags = SheetFlags::new(); 
            loop {
                match reader.read_event(&mut buf) {
                    Ok(Event::Start(ref e)) if e.name() == b"c" => {
                        for a in e.attributes() {
                            let a = a.unwrap(); 
                            match a.key {
                                b"r" => {
                                    // Cell reference
                                    flags.reset(); 
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
                            if let b"si" = a.key {
                                let formula_index: usize = a.unescape_and_decode_value(&reader).unwrap().parse::<usize>().unwrap(); 
                                let (start_cell, formula_text): &(Cell, String) = flags.shared_formulas.get(formula_index).unwrap(); 
                                let base_reference = Reference::from(start_cell.as_tuple()); 
                                let current_cell = Cell::from(flags.current_cell_reference.clone()); 
                                let current_reference = Reference::from(current_cell.as_tuple());
                                let adjusted_formula: Value = Value::Formula(format!("={}", adjust_formula(base_reference, current_reference, formula_text.clone())?)); 
                                let sheet = self.sheets.get_mut(sheet_idx).unwrap(); 
                                let (row, column): (usize, usize) = current_cell.as_tuple(); 
                                sheet.values[[row-1, column-1]].value = adjusted_formula.clone(); 
                                let cell_id = CellId::from((sheet_idx, row, column, 1, 1, Some(false))); 
                                self.dependencies.add_formula(cell_id, &adjusted_formula.to_string(), &self.sheets)?; 
                                flags.reset(); 
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
                                value = Value::from(s.clone()); 
                            } else if flags.is_date {
                                value = Value::from(excel_to_date(cell_text.parse::<f64>().unwrap())); 
                            } else if !cell_text.is_empty() {
                                value = match &*cell_text {
                                    "TRUE" => Value::Bool(true), 
                                    "FALSE" => Value::Bool(false), 
                                    _ => {
                                        Value::Num(cell_text.parse::<f64>().expect("Unable to parse to number"))
                                    }
                                }; 
                            } else {
                                value = Value::Empty; 
                            }
                            let cell = Cell::from(flags.current_cell_reference.clone()); 
                            let (row, column): (usize, usize) = cell.as_tuple(); 
 
                            if value.is_formula() {
                                let cell_id = CellId::from((sheet_idx, row, column, 1, 1, Some(false))); 
                                self.dependencies.add_formula(cell_id, &value.to_string(), &self.sheets)?; 
                            }

                            let sheet = self.sheets.get_mut(sheet_idx).unwrap(); 
                            sheet.values[[row-1, column-1]] = SheetValue { value: value.clone(), calculated: value }; 
                            pb.set_position((row * max_columns + column) as u64); 
                            flags.reset(); 
                        }
                    }, 
                    Ok(Event::Eof) => break, 
                    _ => {
                    } 
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

    pub fn get_mut_sheet_by_name<'a>(&'a mut self, s: &'a str) -> &'a mut Sheet {
        let idx = self.sheets.iter().position(|x| x.name == s).unwrap(); 
        self.get_mut_sheet_by_idx(idx)
    }

    pub fn get_mut_sheet_by_idx(&mut self, idx: usize) -> &mut Sheet {
        self.sheets.get_mut(idx).unwrap()
    }

    pub fn get_sheet_by_name(&self, s: String) -> &Sheet {
        let idx = self.sheets.iter().position(|x| x.name == s.as_str()).unwrap(); 
        self.get_sheet_by_idx(idx)
    }

    pub fn get_sheet_by_idx(&self, idx: usize) -> &Sheet {
        self.sheets.get(idx).unwrap()
    }

    pub fn resolve_str_ref(&self, s: &str) -> Result<Array2<Value>, Error> {
        let expr: Expr = parse_str(s)?; 
        if matches!(expr, Expr::Reference { sheet: _, reference: _}) {
            self.resolve_ref(expr)
        } else {
            panic!("Could not resolve {} to a reference", s); 
        }
    }

    pub fn resolve_ref(&self, expr: Expr) -> Result<Array2<Value>, Error> {
        if let Expr::Reference {sheet, reference} = expr {
            let (mut row, mut col, mut num_rows, mut num_cols) = Reference::from(reference).get_dimensions();
            let sheet: &Sheet = match sheet {
                Some(s) => self.get_sheet_by_name(s), 
                None => self.get_sheet_by_idx(self.current_sheet)
            };
            if num_rows == usize::MAX { 
                num_rows = sheet.values.dim().0; 
                row = 1; // To avoid subtract overflow on row_idx_start
            }
            if num_cols == usize::MAX { 
                num_cols = sheet.values.dim().0; 
                col = 1; // To avoid subtract overflow on col_idx_start
            }
            let row_idx_start: usize = sheet.values.dim().0.min(row-1);
            let row_idx_end: usize = sheet.values.dim().0.min(row+num_rows-1);
            let rows_append: usize = num_rows - (row_idx_end - row_idx_start);
            let col_idx_start: usize = sheet.values.dim().1.min(col-1);
            let col_idx_end: usize = sheet.values.dim().1.min(col+num_cols-1);
            let cols_append: usize = num_cols - (col_idx_end - col_idx_start);
            let mut output: Array2<SheetValue> = sheet.values.slice(s![row_idx_start..row_idx_end, col_idx_start..col_idx_end]).into_owned(); 
            if rows_append > 0 {
                for _ in 0..rows_append {
                    output.push(Axis(0), ArrayView::from(&Array::from_elem(output.dim().1, SheetValue::new()))).unwrap(); 
                }
            }
            if cols_append > 0 {
                for _ in 0..cols_append {
                    output.push(Axis(1), ArrayView::from(&Array::from_elem(output.dim().0, SheetValue::new()))).unwrap(); 
                }
            }
            Ok(output.map(|b| {
                if b.is_calculated() {
                    b.calculated.clone()
                } else {
                    b.value.clone()
                }
            }))
        } else {
            panic!("Can only resolve a reference expression.")
        }
    }

    pub fn calculate_cell(&mut self, cell_id: &CellId, debug: bool) -> Result<(), Error> {
        if ! cell_id.calculated.unwrap_or(true) {
            if debug {
                println!("======= Calculating cell: {}.{}", cell_id.sheet, Reference::from((cell_id.row, cell_id.column))); 
            } 
            let sheet: &Sheet = self.get_sheet_by_idx(cell_id.sheet); 
            let cell_value = &sheet.values[[cell_id.row-1, cell_id.column-1]].value; 
            if let Value::Formula(formula_text) = cell_value.clone() {
                self.current_sheet = cell_id.sheet; 
                let mut chars = formula_text.chars(); // Remove = at beginning
                chars.next();
                let expr: Expr = parse_str(chars.as_str())?; 
                let new_value_result = evaluate_expr_with_context(expr, self, debug);
                match new_value_result {
                    Ok(new_value) => {
                        let sheet: &mut Sheet = self.get_mut_sheet_by_idx(cell_id.sheet); 
                        sheet.values[[cell_id.row-1, cell_id.column-1]].calculated = ensure_non_range(new_value).ensure_single(); 
                        return Ok(()); 
                    }, 
                    Err(e) => {
                        return match e {
                            Error::Volatile(_) => Err(e), 
                            _ => Err(Error::Calculation(cell_id.clone(), Box::new(e)))

                        }; 
                    }
                }; 
            }
        }
        Ok(())
    }

    pub fn is_calculated(&self, expr: Expr) -> bool {
        let value = self.resolve_ref(expr).unwrap(); 
        value.into_raw_vec().iter().all(|x| ! x.is_formula())
    }

    pub fn calculate(&mut self, debug: bool, progress: bool) -> Result<(), Error> {
        loop {
            let mut calculated = true; 
            let order: Vec<CellId> = self.dependencies.get_order(); 
            let pb = match progress {
                true => ProgressBar::new(order.len() as u64), 
                false => ProgressBar::hidden() 
            };
            for cell_id in self.dependencies.get_order().iter_mut() {
                pb.inc(1); 
                match self.calculate_cell(cell_id, debug) {
                    Ok(()) => {
                        cell_id.calculated = Some(true)
                    }, 
                    Err(err) => { 
                        match err {
                            Error::Volatile(new_expr) => {
                                self.dependencies.add_expression(*cell_id, *new_expr, &self.sheets)?; 
                                calculated = false; 
                                break // Recalculate
                            }, 
                            _ => return Err(Error::Calculation(*cell_id, Box::new(err))) 
                        } 
                    }
                }
            }
            if calculated {
                break
            }
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct SheetValue {
    pub value: Value, 
    pub calculated: Value
}

impl From<Value> for SheetValue {
    fn from(v: Value) -> SheetValue {
        SheetValue {
            value: v, 
            calculated: Value::Empty 
        }
    }
}

impl From<(Value, Value)> for SheetValue {
    fn from(v: (Value, Value)) -> SheetValue {
        let (value, calculated) = v; 
        SheetValue { value, calculated }
    }
}

impl SheetValue {
    fn new() -> SheetValue {
        SheetValue { value: Value::Empty, calculated: Value::Empty }
    }

    fn is_calculated(&self) -> bool {
        self.value.is_formula() && ! self.calculated.is_empty()
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Sheet {
    pub name: String,
    pub max_rows: usize, 
    pub max_columns: usize, 
    pub values: Array2<SheetValue>
}

impl From<&str> for Sheet {
    fn from(s: &str) -> Sheet {
        Sheet::from(s.to_string())
    }
}

impl From<String> for Sheet {
    fn from(s: String) -> Sheet {
        Sheet {
            name: s, 
            max_rows: 0, 
            max_columns: 0, 
            values: Array::from_elem((0, 0), SheetValue::new())
        }
    }
}

impl fmt::Display for Sheet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "'{}'!", self.name)
    }
}

#[derive(Debug)]
pub struct SharedString(String); 

#[derive(Default, Debug)]
pub struct Style {
    pub number_format_id: usize, 
    pub apply_number_format: bool 
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
    use crate::workbook::{Sheet, Book};
    use crate::evaluate::value::Value;
    use crate::parser::parse_str; 
    use crate::errors::Error; 
    use ndarray::arr2; 

    fn get_cell<'a>(book: &'a Book, sheet_name: &'a str, row: usize, column: usize) -> Value {
        let sheet: &Sheet = book.get_sheet_by_name(sheet_name.to_string()); 
        sheet.values[[row, column]].value.clone()
    }

    #[test]
    fn test_sheet_names() {
        let mut book = Book::from("assets/data_types.xlsx"); 
        book.load(false).expect("Could not load workbook"); 
        assert_eq!(&book.sheets[0].name, "test 1");
        assert_eq!(&book.sheets[1].name, "test 2");
        assert_eq!(&book.sheets[2].name, "test 3");
    }

    #[test]
    fn test_cells() {
        let mut book = Book::from("assets/data_types.xlsx"); 
        book.load(false).expect("Could not load workbook"); 
        assert_eq!(get_cell(&book, "test 1", 0, 0), Value::from("Text")); 
        assert_eq!(get_cell(&book, "test 1", 1, 0), Value::from("a")); 
        assert_eq!(get_cell(&book, "test 1", 2, 0), Value::from("b")); 
        assert_eq!(get_cell(&book, "test 1", 3, 0), Value::from("c")); 
        assert_eq!(get_cell(&book, "test 1", 1, 4), Value::Formula(String::from("=B2+1"))); 
        assert_eq!(get_cell(&book, "test 1", 2, 4), Value::Formula(String::from("=B3+1"))); 
        assert_eq!(get_cell(&book, "test 1", 3, 4), Value::Formula(String::from("=(B4+1)"))); 
    }

    #[test]
    fn test_resolve_ref() -> Result<(), Error> {
        let mut book = Book::from("assets/basic.xlsx"); 
        book.load(false).expect("Could not load workbook"); 
        book.calculate(false, false)?; 
        assert_eq!(book.resolve_ref(parse_str("Sheet2!B2")?)?, arr2(&[[Value::from(55.0)]])); 
        assert_eq!(book.resolve_ref(parse_str("Sheet2!A1:B2")?)?, arr2(&
            [[Value::Empty, Value::Empty], 
            [Value::Empty, Value::from(55.0)]]
        )); 
        assert_eq!(book.resolve_ref(parse_str("Sheet2!C5:D6")?)?, arr2(&
            [[Value::Empty, Value::Empty], 
            [Value::Empty, Value::Empty]]
        )); 
        assert_eq!(book.resolve_ref(parse_str("Sheet2!B:B")?)?, arr2(&
            [[Value::Empty], 
            [Value::from(55.0)], 
            [Value::Empty]
            ]
        )); 
        Ok(())
    }
}

