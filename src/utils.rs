use crate::evaluate::*;
use crate::parse::Expr;
use crate::excel::ExprParser; 

pub fn evaluate_expr(expr_str: &str) -> String {
    println!("{}", expr_str); 
    let expr : Box<Expr> = ExprParser::new().parse(expr_str).unwrap(); 
    println!("{:?}", expr); 
    format!("{}", Value::from(expr)) 
}


