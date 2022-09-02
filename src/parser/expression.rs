use std::fmt; 

#[derive(Debug, Clone)]
pub enum Expr {
    Num(f32),
    Bool(bool), 
    Error(Error), 
    Cell { sheet: Option<String>, reference: String }, 
    Op(Box<Expr>, Opcode, Box<Expr>),
    Text(String), 
    Func { name: String, args: Vec<Box<Expr>> }, 
    Array(Vec<Box<Expr>>), 
    // TODO: Date
}

impl Expr {
    pub fn is_num(&self) -> bool { matches!(self, Expr::Num(_)) } 
    pub fn is_bool(&self) -> bool { matches!(self, Expr::Bool(_)) }
    pub fn is_error(&self) -> bool { matches!(self, Expr::Error(_)) }
    pub fn is_cell(&self) -> bool { matches!(self, Expr::Cell{ sheet: _, reference: _ }) }
    pub fn is_op(&self) -> bool { matches!(self, Expr::Op(_,_,_)) }
    pub fn is_text(&self) -> bool { matches!(self, Expr::Text(_)) }
    pub fn is_func(&self) -> bool { matches!(self, Expr::Func{ name: _, args: _}) }
    pub fn is_array(&self) -> bool { matches!(self, Expr::Array(_)) }
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Op(a, op, b) => {
                write!(f, "({}{}{})", a, op, b)
            }, 
            Expr::Bool(b) => {
                if *b {
                    write!(f, "TRUE")
                } else {
                    write!(f, "FALSE") 
                }
            }, 
            Expr::Num(n) => {
                write!(f, "{}", n)
            }, 
            Expr::Cell {sheet, reference } => {
                let mut output = String::new();
                if let Some(n) = sheet {
                    output = format!("{}{}!", output, n); // TODO
                }
                output = format!("{}{}", output, reference); 
                write!(f, "{}", output) 
            }, 
            Expr::Error(e) => {
                write!(f, "{}", e)
            }, 
            Expr::Text(s) => {
                write!(f, "\"{}\"", s )
            }, 
            Expr::Func {name, args}  => {
                let mut output: String = format!("{}(", name);
                for (i, arg) in args.iter().enumerate() {
                    if i != 0 {
                        output = format!("{}, {}", output, arg); 
                    } else {
                        output = format!("{}{}", output, arg); 
                    }
                }
                output = format!("{})", output); 
                write!(f, "{}", output) 
            }, 
            Expr::Array(arr) => {
                let mut output: String = String::from("{"); 
                for (i, x) in arr.iter().enumerate() {
                    if i != 0 {
                        output = format!("{}, {}", output, x); 
                    } else {
                        output = format!("{}{}", output, x); 
                    }
                }
                output = format!("{}}}", output); 
                write!(f, "{}", output)
            }

        }
    }
}

#[derive(Debug, Clone)]
pub enum Opcode {
    Colon,
    SemiColon,
    Comma, 
    Space, 
    Exponent, 
    Multiply,
    Divide,
    Add,
    Subtract,
    Concat, 
    Equal, 
    NotEqual, 
    LessThan, 
    LessThanOrEqual, 
    GreaterThan, 
    GreaterThanOrEqual, 
    Percent
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Opcode::Colon => ":",
            Opcode::SemiColon => ";", 
            Opcode::Comma => ",", 
            Opcode::Space => " ", 
            Opcode::Exponent => "^", 
            Opcode::Multiply => "*", 
            Opcode::Divide => "/", 
            Opcode::Add => "+", 
            Opcode::Subtract => "-", 
            Opcode::Concat => "&", 
            Opcode::Equal => "=", 
            Opcode::NotEqual => "<>", 
            Opcode::LessThan => "<", 
            Opcode::LessThanOrEqual => "<=", 
            Opcode::GreaterThan => ">", 
            Opcode::GreaterThanOrEqual => ">=", 
            Opcode::Percent => "%"
        })
    }
}

#[derive(Debug, Clone)]
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
        write!(f, "{}", match self {
            Error::Null => "#NULL!",
            Error::Div => "#DIV/0!", 
            Error::Value => "#VALUE!", 
            Error::Ref => "#REF!", 
            Error::Name => "#NAME?", 
            Error::Num => "#NUM!", 
            Error::NA => "#N/A", 
            Error::GettingData => "#GETTING_DATA"
        })
    }
}
