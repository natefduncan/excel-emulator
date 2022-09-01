use clap::{Parser, Subcommand};
use excel_lib::workbook::Book; 

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    path: String, 

    #[clap(subcommand)]
    command: Option<Commands>
}

#[derive(Subcommand)]
enum Commands {
    Load, 
    Deps,
}

fn main() {
    let cli = Cli::parse();
    let mut book: Book = Book::from(cli.path); 
    match &cli.command {
        Some(Commands::Load) => { book.load().expect("Could not load workbook.")}, 
        Some(Commands::Deps) => { 
            book.load().expect("Could not load workbook."); 
            println!("{}", book.dependencies); 
        }, 
        _ => {}
    }
}
