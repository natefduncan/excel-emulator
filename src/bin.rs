use clap::{Parser, Subcommand};
use excel_lib::{
    //workbook::Book, 
    parser::{
        ast::Expr, 
        parse_str
    }, 
    evaluate::evaluate_str, 
    errors::Error
}; 
use std::io;
use std::io::prelude::*;
use std::io::Write;

#[derive(Parser)]
#[clap(author, version, about, long_about = "Parse excel file and run logic in rust")]
struct Cli {
    //#[clap(value_parser)]
    //path: String, 

    #[clap(subcommand)]
    command: Option<Commands>, 

    #[clap(short, long)]
    progress: bool, 

    #[clap(short, long)]
    debug: bool 
}

#[derive(Subcommand)]
enum Commands {
    //Load, 
    //Deps,
    //Order,
    //Sheets, 
    //Get {
        //#[clap(value_parser)]
        //range: String 
    //}, 
    //Calculate {
        //#[clap(value_parser)]
        //range: String 
    //}, 
    Repl
}

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    //let mut book: Book = Book::from(cli.path); 
    //book.load(cli.progress)?; 
    match &cli.command {
        //Some(Commands::Load) => { book.load(cli.progress)?}, 
        //Some(Commands::Deps) => { 
            //println!("{}", book.dependencies); 
        //}, 
        //Some(Commands::Order) => {
            //for o in book.dependencies.get_order().iter() {
                //println!("{:?}", o); 
            //}
        //}, 
        //Some(Commands::Sheets) => {
            //println!("{:?}", book.sheets.iter().map(|x| x.name.clone()).collect::<Vec<String>>()); 
        //}, 
        //Some(Commands::Get {range}) => {
            //let expr: Expr = parse_str(range)?;
            //if matches!(expr, Expr::Reference { sheet: _, reference: _} ) {
				//println!("{}", &book.resolve_ref(expr)?); 
            //} else {
                //panic!("Could not resolve {} to a reference.", range); 
            //}
        //}, 
        //Some(Commands::Calculate {range}) => {
            //book.calculate(cli.debug, cli.progress)?; 
            //println!("{:?}", book.resolve_str_ref(range)); 
        //}
        Some(Commands::Repl) => {
            loop {
                let mut cmd_str = String::new(); 
                print!("> "); 
                io::stdout().flush().unwrap();
                io::stdin().read_line(&mut cmd_str).unwrap(); 
                match cmd_str.as_str() {
                    ":q" => { break }, 
                    c => println!("{}", evaluate_str(c)?)
                }
            }
        }, 
        _ => {}
    }
    Ok(())
}
