use crate::parser::{
    ast::{Literal, Prefix, Infix, Expr}, 
    parse
}; 
use crate::function::*; 
use crate::lexer::{
    token::Tokens, 
    Lexer, 
}; 
use crate::workbook::Book; 

pub mod value; 
use crate::evaluate::value::Value; 

pub fn evaluate_str(s: &str) -> Value {
    let (_, t) = Lexer::lex_tokens(s.as_bytes()).unwrap();
    let tokens = Tokens::new(&t); 
    let (_, expr) = parse(tokens).unwrap(); 
    evaluate_expr(expr)
}

pub fn evaluate_expr(expr: Expr) -> Value {
     match expr {
        Expr::Func {name, args} => {
            let arg_values: Vec<Value> = args.into_iter().map(evaluate_expr).collect::<Vec<Value>>(); 
            get_function_value(&name, arg_values)
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
				Prefix::Plus => Value::from(evaluate_expr(*box_expr).as_num().abs()),
				Prefix::Minus => evaluate_expr(*box_expr) * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
			match i {
				Infix::Plus => evaluate_expr(*a) + evaluate_expr(*b), 
				Infix::Minus => evaluate_expr(*a) - evaluate_expr(*b), 
				Infix::Multiply => evaluate_expr(*a) * evaluate_expr(*b), 
				Infix::Divide => evaluate_expr(*a) / evaluate_expr(*b), 
				Infix::Exponent => Exponent {a: evaluate_expr(*a), b: evaluate_expr(*b)}.evaluate(), 
				Infix::NotEqual => Value::from(evaluate_expr(*a) != evaluate_expr(*b)), 
				Infix::Equal => Value::from(evaluate_expr(*a) == evaluate_expr(*b)), 
				Infix::LessThan => Value::from(evaluate_expr(*a) < evaluate_expr(*b)), 
				Infix::LessThanEqual => Value::from(evaluate_expr(*a) <= evaluate_expr(*b)), 
				Infix::GreaterThan => Value::from(evaluate_expr(*a) > evaluate_expr(*b)), 
				Infix::GreaterThanEqual => Value::from(evaluate_expr(*a) >= evaluate_expr(*b)), 
				_ => panic!("Infix {:?} does not convert to a value.", i) 
			}
		}, 
		Expr::Array(x) => Value::Array(x.into_iter().map(evaluate_expr).collect::<Vec<Value>>()), 
		_ => panic!("Expression {:?} does not convert to a value.", expr)  
	}
}

pub fn evaluate_expr_with_context(expr: Expr, book: &Book) -> Value {
    match expr {
        Expr::Reference { sheet: _, reference: _ } => {
            Value::from(book.resolve_ref(expr.clone()))
		}, 
        Expr::Func {name, args} => {
            let arg_values: Vec<Value> = args.into_iter().map(|x| evaluate_expr_with_context(x, book)).collect::<Vec<Value>>(); 
            get_function_value(&name, arg_values)
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
				Prefix::Plus => Value::from(evaluate_expr_with_context(*box_expr, book).as_num().abs()),
				Prefix::Minus => evaluate_expr_with_context(*box_expr, book) * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
			match i {
				Infix::Plus => evaluate_expr_with_context(*a, book) + evaluate_expr_with_context(*b, book), 
				Infix::Minus => evaluate_expr_with_context(*a, book) - evaluate_expr_with_context(*b, book), 
				Infix::Multiply => evaluate_expr_with_context(*a, book) * evaluate_expr_with_context(*b, book), 
				Infix::Divide => evaluate_expr_with_context(*a, book) / evaluate_expr_with_context(*b, book), 
				Infix::Exponent => Exponent {a: evaluate_expr_with_context(*a, book), b: evaluate_expr_with_context(*b, book)}.evaluate(), 
				Infix::NotEqual => Value::from(evaluate_expr_with_context(*a, book) != evaluate_expr_with_context(*b, book)), 
				Infix::Equal => Value::from(evaluate_expr_with_context(*a, book) == evaluate_expr_with_context(*b, book)), 
				Infix::LessThan => Value::from(evaluate_expr_with_context(*a, book) < evaluate_expr_with_context(*b, book)), 
				Infix::LessThanEqual => Value::from(evaluate_expr_with_context(*a, book) <= evaluate_expr_with_context(*b, book)), 
				Infix::GreaterThan => Value::from(evaluate_expr_with_context(*a, book) > evaluate_expr_with_context(*b, book)), 
				Infix::GreaterThanEqual => Value::from(evaluate_expr_with_context(*a, book) >= evaluate_expr_with_context(*b, book)), 
				_ => panic!("Infix {:?} does not convert to a value.", i) 
			}
		}, 
		Expr::Array(x) => Value::Array(x.into_iter().map(|e| evaluate_expr_with_context(e, book)).collect::<Vec<Value>>()), 
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
        assert_eq!(evaluate_str(" 1 + 1 "), Value::from(2.0));
        assert_eq!(evaluate_str(" 1 - 1 "), Value::from(0.0)); 
        assert_eq!(evaluate_str(" 2 * 2 "), Value::from(4.0)); 
        assert_eq!(evaluate_str(" (2 + 1) * 2 "), Value::from(6.0)); 
        assert_eq!(evaluate_str(" 8 / 4 "), Value::from(2.0)); 
        assert_eq!(evaluate_str(" 8^2 "), Value::from(64.0)); 
    }

    #[test]
    fn test_conditionals() {
        let book = &Book::new(); 
        assert_eq!(evaluate_str(" 1=1 "), Value::from(true)); 
    }

    #[test]
    fn test_formula() {
        let book = &Book::new(); 
        assert_eq!(evaluate_str(" SUM(1, 1) "), Value::from(2.0)); 
        assert_eq!(evaluate_str(" SUM(SUM(1, 2), 1) "), Value::from(4.0)); 
    }
}

