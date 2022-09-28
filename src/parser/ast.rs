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
        sheet: Option<String>, 
        reference: String 
    }, 
	Array(Vec<Expr>),
    Error(Error)
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Literal(l) => write!(f, "{}", l), 
            Expr::Prefix(p, e) => write!(f, "{}{}", p, e), 
            Expr::Infix(p, a, b) => write!(f, "({}{}{})", a, p, b), 
            Expr::Func{name, args} => {
                let output = format!("{}({})", name, exprs_string(args));
                write!(f, "{}", output)
            }, 
            Expr::Reference{sheet, reference} => {
                match sheet {
                    Some(s) => {
                        write!(f, "{}!{}", s, reference)
                    }, 
                    None => write!(f, "{}", reference)
                }
            }, 
            Expr::Array(arr) => write!(f, "{{{}}}", exprs_string(arr)), 
            Expr::Error(e) => write!(f, "{}", e)
        }
    }
}

fn exprs_string(v: &Vec<Expr>) -> String {
    let mut output = String::new(); 
    for (i, arg) in v.iter().enumerate() {
        if i == v.len()-1 {
            output.push_str(&arg.to_string());
        } else {
            output.push_str(&arg.to_string());
            output.push_str(", ");
        }
    }
    output
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

#[derive(Debug, PartialEq, Eq, Clone)]
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

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Null => write!(f, "#NULL!"), 
            Error::Div => write!(f, "#DIV/0!"), 
            Error::Value => write!(f, "#VALUE!"),
            Error::Ref => write!(f, "#REF!"),
            Error::Name => write!(f, "#NAME!"), 
            Error::Num => write!(f, "#NUM!"), 
            Error::NA => write!(f, "#N/A!"), 
            Error::GettingData => write!(f, "#GETTING_DATA")
        }
    }
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
            Literal::Number(x) => write!(f, "{}", x), 
            Literal::Boolean(b) => {
                if *b {
                    write!(f, "TRUE")
                } else {
                    write!(f, "FALSE")
                }
            },
            Literal::Text(s) => write!(f, "{}", s)
        }
    }
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Prefix {
    Plus,
    Minus,
}

impl fmt::Display for Prefix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Prefix::Plus => write!(f, "+"), 
            Prefix::Minus => write!(f, "-"), 
        }
    }
}
 

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
    Exponent,
    Ampersand,
    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,
}

impl fmt::Display for Infix {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Infix::Plus => write!(f, "+"), 
            Infix::Minus => write!(f, "-"), 
            Infix::Divide => write!(f, "/"), 
            Infix::Multiply => write!(f, "*"),
            Infix::Exponent => write!(f, "^"), 
            Infix::Ampersand => write!(f, "&"), 
            Infix::Equal => write!(f, "="), 
            Infix::NotEqual => write!(f, "<>"), 
            Infix::GreaterThanEqual => write!(f, ">="), 
            Infix::LessThanEqual => write!(f, "<="), 
            Infix::GreaterThan => write!(f, ">"), 
            Infix::LessThan => write!(f, "<") 
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Debug, Clone)]
pub enum Precedence {
    Lowest, 
    Comparison, 
    Concat, 
    PlusMinus, 
    MultDiv, 
    Exponent, 
    Percent, 
}
