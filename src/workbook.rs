use quick_xml::events::BytesStart;
use zip::read::{ZipArchive, ZipFile};
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
use anyhow::Result; 
pub type ZipType = ZipArchive<File>; 

pub struct Book {
    path: String,
    zip: ZipType, 
    sheets: Vec<Sheet>, 
    shared_strings: Vec<SharedString>, 
    styles: Vec<Style>
}

impl From<String> for Book {
    fn from(s: String) -> Self {
        let zip = Self::zip_from_path(&s); 
        Book { path: s, zip, sheets: vec![], shared_strings: vec![], styles: vec![] }
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
                            self.shared_strings.push(
                                SharedString(Box::new(Self::decode_text_event(&reader, e)))
                            )
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

    pub fn load_sheets(&mut self) -> Result<()> { Ok(()) }
    pub fn load_sheet(&mut self, sheet_name: &str) -> Result<()> {
        Ok(())
    }

    pub fn zip_from_path(path: &str) -> ZipType {
        println!("{:?}", path); 
        let file: File = File::open(path).expect("Unable to find file"); 
        zip::ZipArchive::new(file).expect("Unable to create zip") 
    }

    // fn get_shared_string_by_index(&self, index: usize) -> &Box<String> {
        // if let Some(SharedString(s)) = self.shared_strings.get(index) {
            // s
        // } else {
            // panic!("Shared string table does not have index {}", index)
        // }
    // }

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
}

#[derive(Debug)]
pub struct Sheet(String); 
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

#[cfg(test)]
mod tests {
    use crate::workbook::Book;

    #[test]
    fn sheet_names() {
        let mut book = Book::from("assets/data_types.xlsx"); 
        book.load().expect("Could not load workbook"); 
        assert_eq!(&book.sheets[0].name(), "test 1");
        assert_eq!(&book.sheets[1].name(), "test 2");
        assert_eq!(&book.sheets[2].name(), "test 3");
    }
}

