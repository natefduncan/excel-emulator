use ndarray::Array2; use crate::parser::{
    ast::{Literal, Prefix, Infix, Expr}, 
    parse
}; 
use crate::function::*; 
use crate::lexer::{
    token::Tokens, 
    Lexer, 
}; 
use crate::workbook::Book; 
use crate::reference::Reference;

pub mod value; 
use crate::evaluate::value::Value; 

pub fn evaluate_str(s: &str, book: &Book) -> Value {
    let (_, t) = Lexer::lex_tokens(s.as_bytes()).unwrap();
    let tokens = Tokens::new(&t); 
    let (_, expr) = parse(tokens).unwrap(); 
    evaluate_expr(expr, book)
}

pub fn evaluate_expr(expr: Expr, book: &Book) -> Value {
    match expr {
        Expr::Reference{ sheet, reference } => Value::Ref { sheet, reference: Reference::from(reference) }, 
        Expr::Func {name, args} => {
            Value::from(0.0)
        },
		Expr::Literal(lit) => {
			match lit {
				Literal::Number(f) => Value::from(f), 
				Literal::Boolean(b) => Value::from(b), 
				Literal::Text(s) => Value::from(s)
			}
		},
		Expr::Prefix(p, box_expr) => { 
			match p {
				Prefix::Plus => Value::from(evaluate_expr(*box_expr, book).as_num().abs()),
				Prefix::Minus => evaluate_expr(*box_expr, book) * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
			match i {
				Infix::Plus => evaluate_expr(*a, book) + evaluate_expr(*b, book), 
				Infix::Minus => evaluate_expr(*a, book) - evaluate_expr(*b, book), 
				Infix::Multiply => evaluate_expr(*a, book) * evaluate_expr(*b, book), 
				Infix::Divide => evaluate_expr(*a, book) / evaluate_expr(*b, book), 
				Infix::Exponent => Exponent {a: evaluate_expr(*a, book), b: evaluate_expr(*b, book)}.evaluate(), 
				Infix::NotEqual => Value::from(evaluate_expr(*a, book) != evaluate_expr(*b, book)), 
				Infix::Equal => Value::from(evaluate_expr(*a, book) == evaluate_expr(*b, book)), 
				Infix::LessThan => Value::from(evaluate_expr(*a, book) < evaluate_expr(*b, book)), 
				Infix::LessThanEqual => Value::from(evaluate_expr(*a, book) <= evaluate_expr(*b, book)), 
				Infix::GreaterThan => Value::from(evaluate_expr(*a, book) > evaluate_expr(*b, book)), 
				Infix::GreaterThanEqual => Value::from(evaluate_expr(*a, book) >= evaluate_expr(*b, book)), 
				_ => panic!("Infix {:?} does not convert to a value.", i) 
			}
		}, 
		Expr::Array(x) => Value::Array(x.into_iter().map(|e| evaluate_expr(e, book)).collect::<Vec<Value>>()), 
		_ => panic!("Expression {:?} does not convert to a value.", expr)  
	}
}



#[cfg(test)]
mod tests {
	use crate::evaluate::evaluate_str;
    use crate::evaluate::value::Value; 
    use crate::workbook::Book; 

    #[test]
    fn test_op_codes() {
        let book = &Book::new(); 
        assert_eq!(evaluate_str(" 1 + 1 ", book), Value::from(2.0));
        assert_eq!(evaluate_str(" 1 - 1 ", book), Value::from(0.0)); 
        assert_eq!(evaluate_str(" 2 * 2 ", book), Value::from(4.0)); 
        assert_eq!(evaluate_str(" (2 + 1) * 2 ", book), Value::from(6.0)); 
        assert_eq!(evaluate_str(" 8 / 4 ", book), Value::from(2.0)); 
        assert_eq!(evaluate_str(" 8^2 ", book), Value::from(64.0)); 
    }

    #[test]
    fn test_conditionals() {
        let book = &Book::new(); 
        assert_eq!(evaluate_str(" 1=1 ", book), Value::from(true)); 
    }

    #[test]
    fn test_formula() {
        let book = &Book::new(); 
        assert_eq!(evaluate_str(" SUM(1, 1) ", book), Value::from(2.0)); 
        assert_eq!(evaluate_str(" SUM(SUM(1, 2), 1) ", book), Value::from(4.0)); 
    }
}

