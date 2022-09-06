use crate::parser::parse; 
use crate::lexer::{
    token::Tokens, 
    Lexer, 
}; 
pub mod value; 
use crate::evaluate::value::Value; 

pub fn evaluate_str(s: &str) -> Value {
    let (_, t) = Lexer::lex_tokens(s.as_bytes()).unwrap();
    let tokens = Tokens::new(&t); 
    let (_, expr) = parse(tokens).unwrap(); 
    Value::from(expr)
}

#[cfg(test)]
mod tests {
	use crate::evaluate::evaluate_str;
    use crate::evaluate::value::Value; 

    #[test]
    fn test_op_codes() {
        assert_eq!(evaluate_str(" 1 + 1 "), Value::from(2.0));
        assert_eq!(evaluate_str(" 1 - 1 "), Value::from(0.0)); 
        assert_eq!(evaluate_str(" 2 * 2 "), Value::from(4.0)); 
        assert_eq!(evaluate_str(" (2 + 1) * 2 "), Value::from(6.0)); 
        assert_eq!(evaluate_str(" 8 / 4 "), Value::from(2.0)); 
        assert_eq!(evaluate_str(" 8^2 "), Value::from(64.0)); 
    }

    #[test]
    fn test_conditionals() {
        assert_eq!(evaluate_str(" 1=1 "), Value::from(true)); 
    }

    #[test]
    fn test_formula() {
        assert_eq!(evaluate_str(" SUM(1, 1) "), Value::from(2.0)); 
        assert_eq!(evaluate_str(" SUM(SUM(1, 2), 1) "), Value::from(4.0)); 
    }
}

