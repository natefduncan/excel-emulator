use std::fmt; 

#[derive(Debug)]
pub enum Expr {
    Num(f32),
    Bool(bool), 
    Error(Error), 
    Cell { sheet: Option<String>, reference: String }, 
    Op(Box<Expr>, Opcode, Box<Expr>),
    Text(String), 
    Func { name: String, args: Vec<Box<Expr>> }, 
    Array(Vec<Box<Expr>>), 
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Op(a, op, b) => {
                write!(f, "({}{}{})", a, op, b)
            }, 
            Expr::Bool(b) => {
                if *b {
                    write!(f, "{}", "TRUE")
                } else {
                    write!(f, "{}", "FALSE") 
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

#[derive(Debug)]
pub enum Opcode {
    Colon,
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
    GreatThanOrEqual, 
    Percent
}

impl fmt::Display for Opcode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self {
            Opcode::Colon => ":",
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
            Opcode::GreatThanOrEqual => ">=", 
            Opcode::Percent => "%"
        })
    }
}

#[derive(Debug)]
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


#[cfg(test)]
mod tests {
    use crate::excel::*; 
    fn parse_expr(expr: &str) -> String {
        println!("{}", expr); 
        println!("{:?}", ExprParser::new().parse(expr).unwrap()); 
        format!("{}", ExprParser::new().parse(expr).unwrap())
    }

    #[test]
    fn test_num() {
        assert_eq!(&parse_expr(" 1 "), "1"); 
        assert_eq!(&parse_expr(" 150 "), "150"); 
    }

    #[test]
    fn test_operators() {
        assert_eq!(&parse_expr("1 + 1"), "(1+1)"); 
        assert_eq!(&parse_expr("1 - 1"), "(1-1)");
        assert_eq!(&parse_expr("1 / 1"), "(1/1)");
        assert_eq!(&parse_expr("1 * 1"), "(1*1)");
        assert_eq!(&parse_expr("1 ^ 1"), "(1^1)");
        assert_eq!(&parse_expr("1 = 1"), "(1=1)");
        assert_eq!(&parse_expr("1 < 1"), "(1<1)");
        assert_eq!(&parse_expr("1 <= 1"), "(1<=1)");
        assert_eq!(&parse_expr("1 > 1"), "(1>1)");
        assert_eq!(&parse_expr("1 >= 1"), "(1>=1)");
        assert_eq!(&parse_expr("1 <> 1"), "(1<>1)");
        assert_eq!(&parse_expr("1 % 1"), "(1%1)");
        assert_eq!(&parse_expr("22 * 44 + 66"), "((22*44)+66)");
    }

    #[test] 
    fn test_errors() {
        assert_eq!(&parse_expr("#NULL!"), "#NULL!");
        assert_eq!(&parse_expr("#DIV/0!"), "#DIV/0!");
        assert_eq!(&parse_expr("#VALUE!"), "#VALUE!");
        assert_eq!(&parse_expr("#REF!"), "#REF!");
        assert_eq!(&parse_expr("#NAME?"), "#NAME?");
        assert_eq!(&parse_expr("#NUM!"), "#NUM!");
        assert_eq!(&parse_expr("#N/A"), "#N/A");
        assert_eq!(&parse_expr("#GETTING_DATA"), "#GETTING_DATA");
    }

    #[test]
    fn test_cell() {
        assert_eq!(&parse_expr(" Sheet!A1 "), "Sheet!A1");
        assert_eq!(&parse_expr(" 'Sheet 1'!A1 "), "'Sheet 1'!A1");
        assert_eq!(&parse_expr(" 'Sheet 1':'Sheet 2'!A1 "), "'Sheet 1':'Sheet 2'!A1");
        assert_eq!(&parse_expr(" Sheet1:Sheet2!A1 "), "Sheet1:Sheet2!A1");
        assert_eq!(&parse_expr(" Sheet1!A1:A2 "), "Sheet1!A1:A2");
        assert_eq!(&parse_expr(" Sheet1!$A$1:A2 "), "Sheet1!$A$1:A2");
        assert_eq!(&parse_expr(" A1 "), "A1");
    }

    #[test]
    fn test_bool() {
        assert_eq!(&parse_expr(" TRUE "), "TRUE"); 
        assert_eq!(&parse_expr(" FALSE "), "FALSE"); 
    }

    #[test]
    fn test_text() {
        assert_eq!(&parse_expr(" \" TEST \" "), "\"TEST\"");
    }

    #[test]
    fn test_function() {
        assert_eq!(&parse_expr(" test('a', 'b') "), "test(\"a\", \"b\")"); 
    }

    #[test]
    fn test_array() {
        assert_eq!(&parse_expr(" {1, 2, 3, 4} "), "{1, 2, 3, 4}"); 
    }

    #[test]
    fn test_mix() {
        assert_eq!(&parse_expr("test({1, 2, 3, 4}, 1, 'a')"), "test({1, 2, 3, 4}, 1, \"a\")")
    }
}
