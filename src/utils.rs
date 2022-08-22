use chrono::{NaiveDate, Duration}; 

use crate::evaluate::*;
use crate::parse::Expr;
use crate::excel::ExprParser; 

pub fn evaluate_expr(expr_str: &str) -> String {
    println!("{}", expr_str); 
    let expr : Box<Expr> = ExprParser::new().parse(expr_str).unwrap(); 
    println!("{:?}", expr); 
    format!("{}", Value::from(expr)) 
}

pub fn excel_to_date(serial: f64)  -> NaiveDate {
    let start_date = NaiveDate::from_ymd(1899, 12, 30); 
    let mut duration = Duration::days(serial as i64); 
    let frac = (serial - serial.floor()) as i64; 
    if frac > 0 {
        duration = duration.checked_add(&Duration::hours(24 * frac)).expect("Could not add two dates together"); 
    }
    start_date.checked_add_signed(duration).unwrap()
}


