use quick_xml::events::BytesStart;
use zip::read::{ZipArchive, ZipFile};
use std::fs::File;
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
    sheet: Vec<Sheet>, 
    shared_strings: Vec<SharedString>, 
    styles: Vec<Style>
}

impl Default for Book {
    fn default() -> Self {
        
        Book {
            path: String::new(), 
            zip: Self::zip_from_path(""), //This will fail 
            sheet: vec![], 
            shared_strings: vec![], 
            styles: vec![]
        }
    }
}

impl From<String> for Book {
    fn from(s: String) -> Self {
        let zip = Self::zip_from_path(&s); 
        Book { path: s, zip, ..Default::default() }
    }
}

impl From<&str> for Book {
    fn from(s: &str) -> Self {
        Book::from(s.to_string())
    }
}

impl Book {
    fn load(&mut self) -> Result<()> {
        self.load_sheets(); 
        self.load_shared_strings()?; 
        self.load_styles(); 
        Ok(())
    }

    fn load_shared_strings(&mut self) -> Result<()> {
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
            }
        }
       Ok(())
    }

    fn load_styles(&mut self) {
        let mut buf = Vec::new();
        if let Ok(f) = self.zip.by_name("xl/sharedStrings.xml") {
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
                    _ => {}
                }
            }
        }
    }

    fn load_sheets(&mut self) { }
    fn load_sheet(&mut self, sheet_name: &str) -> Result<()> {
        Ok(())
    }

    fn zip_from_path(path: &str) -> ZipType {
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

    fn decode_text_event(reader: &Reader<BufReader<ZipFile>>, e: &BytesText) -> String {
        e.unescape_and_decode(reader).unwrap()
    }

    fn decode_attribute_usize(reader: &Reader<BufReader<ZipFile>>, a: Attribute) -> usize {
        a.unescape_and_decode_value(reader)
        .unwrap()
        .parse::<usize>()
        .unwrap() 
    }

    fn decode_style(reader: &Reader<BufReader<ZipFile>>, e: &BytesStart) -> Style {
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

pub struct Sheet(String); 
pub struct SharedString(Box<String>); 
pub struct Style {
    number_format_id: usize, 
    apply_number_format: bool 
}

impl Default for Style {
    fn default() -> Style {
        Style { number_format_id: 0, apply_number_format: false }
    }
}

impl Style {
    fn new() -> Style {
        Default::default()
    }

}
