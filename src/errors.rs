use thiserror::Error; 
use crate::dependency::CellId; 
use crate::parser::ast::Expr; 

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to calculate cell {0} with error {1}")]
    Calculation(CellId, Box<Error>), 

    #[error("Function {0} is not supported")]
    FunctionNotSupport(String),

    #[error("Unable to parse str {0}")]
    UnableToParse(String), 

    #[error("Unable to lex str {0}")]
    UnableToLex(String), 

    #[error("Dependency tree changed.")]
    Volatile(Box<Expr>)
}
