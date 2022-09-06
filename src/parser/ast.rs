use std::fmt;  

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Prefix(Prefix, Box<Expr>),
    Infix(Infix, Box<Expr>, Box<Expr>),
	Func {
        name: String, 
        args: Vec<Expr>
    },
    Reference {
        sheet: String, 
        reference: String 
    }, 
	Array(Vec<Expr>),
    Error(Error)
}

#[derive(Debug, PartialEq, Clone)]
pub enum Error {
    Null,
    Div, 
    Value,
    Ref, 
    Name, 
    Num, 
    NA, 
    GettingData
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Number(f64),
    Boolean(bool),
    Text(String),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Number(x) => write!(f, "{}", x.to_string()), 
            Literal::Boolean(b) => {
                if *b {
                    write!(f, "{}", "TRUE")
                } else {
                    write!(f, "{}", "FALSE")
                }
            },
            Literal::Text(s) => write!(f, "{}", s)
        }
    }
}

#[derive(PartialEq, Debug, Clone)]
pub enum Prefix {
    Plus,
    Minus,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,
}
