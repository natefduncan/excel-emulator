use clap::{Parser, Subcommand};
use excel_lib::{
    workbook::Book, 
    parser::{
        ast::Expr, 
        parse_str
    }
}; 

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
    Order,
    Get {
        #[clap(value_parser)]
        range: String 
    }, 
}

fn main() {
    let cli = Cli::parse();
    let mut book: Book = Book::from(cli.path); 
    book.load().expect("Could not load workbook."); 
    match &cli.command {
        Some(Commands::Load) => { book.load().expect("Could not load workbook.")}, 
        Some(Commands::Deps) => { 
            println!("{}", book.dependencies); 
        }, 
        Some(Commands::Order) => {
            println!("{:?}", book.dependencies.get_order()); 
        }, 
        Some(Commands::Get {range}) => {
            // book.calculate(); 
            let expr: Expr = parse_str(&range);
            if matches!(expr, Expr::Reference { sheet: _, reference: _} ) {
				println!("{}", &book.resolve_ref(expr)); 
            } else {
                panic!("Could not resolve {} to a reference.", range); 
            }
        }, 
        _ => {}
    }
}
