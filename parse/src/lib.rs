#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub excel); 

pub mod ast {
    use std::fmt; 

    #[derive(Debug)]
    pub enum Expr {
        Num(i32),
        Op(Box<Expr>, Opcode, Box<Expr>),
        Str(String), 
    }

    impl fmt::Display for Expr {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            match self {
                Expr::Op(a, op, b) => {
                    write!(f, "({}{}{})", a, op, b)
                }, 
                Expr::Num(n) => {
                    write!(f, "{}", n)
                }, 
                Expr::Str(s) => {
                    write!(f, "{}", s) 
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
}

#[cfg(test)]
mod tests {
    use crate::excel::*; 
    fn parse_expr(expr: &str) -> String {
        println!("{}", expr); 
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
}
