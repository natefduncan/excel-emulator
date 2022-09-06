use std::fmt;  

#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Prefix(Prefix, Box<Expr>),
    Infix(Infix, Box<Expr>, Box<Expr>),
	Func {
        name: String, 
        args: Vec<Box<Expr>>
    },
    Reference {
        sheet: Option<String>, 
        reference: String 
    }, 
	Array(Vec<Box<Expr>>),
    Error(Error)
}

impl From<f64> for Expr {
    fn from(f: f64) -> Expr {
        Expr::Literal(Literal::from(f))    }
}

impl From<String> for Expr {
    fn from(s: String) -> Expr {
        Expr::Literal(Literal::from(s))
    }
}

impl From<&str> for Expr {
    fn from(s: &str) -> Expr {
        Expr::from(s.to_string())
    }
}

impl From<bool> for Expr {
    fn from(b: bool) -> Expr {
        Expr::Literal(Literal::from(b))
    }
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

impl From<f64> for Literal {
    fn from(f: f64) -> Literal {
        Literal::Number(f)
    }
}

impl From<String> for Literal {
    fn from(s: String) -> Literal {
        Literal::Text(s)
    }
}

impl From<&str> for Literal {
    fn from(s: &str) -> Literal {
        Literal::Text(s.to_string())
    }
}

impl From<bool> for Literal {
    fn from(b: bool) -> Literal {
        Literal::Boolean(b)
    }
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
    Exponent,
    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,
}
