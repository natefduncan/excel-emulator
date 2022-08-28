use chrono::{NaiveDate, Duration}; 

use crate::evaluate::*;
use crate::parse::Expr;
use crate::excel::ExprParser; 
use crate::reference::Reference; 

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

pub fn adjust_formula(
    base_reference: Reference,
    current_reference: Reference,
    formula_text: String,
) -> String {
    let row_offset: i32 = current_reference.row() as i32 - base_reference.row() as i32;
    let column_offset: i32 = current_reference.column() as i32 - base_reference.column() as i32;
	let mut expression: Box<Expr> = ExprParser::new().parse(&formula_text).unwrap(); 
    adjust_expression(row_offset, column_offset, &mut expression); 
	format!("{}", expression) 
}

pub fn adjust_expression(
    row_offset: i32, 
    column_offset: i32, 
    expression: &mut Expr
) {
    match *expression {
        Expr::Cell { sheet: _, ref mut reference } => {
            let mut r = Reference::from(reference.to_string());
			r.offset((row_offset, column_offset));
			*reference = r.to_string(); 
        }, 
        Expr::Op(ref mut a, _, ref mut b) => {
            adjust_expression(row_offset, column_offset, a); 
            adjust_expression(row_offset, column_offset, b); 
        }, 
        Expr::Func { name: _, ref mut args } => {
            for arg in args.iter_mut() {
                adjust_expression(row_offset, column_offset, arg); 
            } 
        }, 
        Expr::Array(ref mut arr) => {
            for a in arr.iter_mut() {
                adjust_expression(row_offset, column_offset, a); 
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use crate::reference::Reference; 
    use crate::utils::adjust_formula; 
    
    #[test]
    fn test_adjust_formula() {
       let base_reference = Reference::from((1, 1));
       let current_reference = Reference::from((2, 1));
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("Sheet1!A1")), &"Sheet1!A2"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("A1+A2")), &"(A2+A3)"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("SUM(A1+A2)")), &"SUM((A2+A3))"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("{A1, A2}")), &"{A2, A3}"); 
    }

    #[test]
    fn test_adjust_formula_anchored() {
       let base_reference = Reference::from((1, 1));
       let current_reference = Reference::from((2, 2));
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("Sheet1!$A$1")), &"Sheet1!$A$1"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("$A$1+A2")), &"($A$1+B3)"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("SUM(A$1+A$2)")), &"SUM((B$1+B$2))"); 
       assert_eq!(&adjust_formula(base_reference, current_reference, String::from("{A1, A2}")), &"{B2, B3}"); 
    }
}
