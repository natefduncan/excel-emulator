use thiserror::Error; 

#[derive(Error, Debug)]
pub enum Error {
    #[error("Unable to calculate cell {0}")]
    Calculation(String), 

    #[error("Function {0} is not supported")]
    FunctionNotSupport(String),

    #[error("Unable to parse str {0}")]
    UnableToParse(String), 

    #[error("Unable to lex str {0}")]
    UnableToLex(String)
}
