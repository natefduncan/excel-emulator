use chrono::{NaiveDate, Duration}; 

use crate::parser::{
    parse_str, 
    ast::Expr
}; 
//use crate::reference::Reference; 
use crate::errors::Error; 

pub fn excel_to_date(serial: f64)  -> NaiveDate {
    let start_date = NaiveDate::from_ymd(1899, 12, 30); 
    let mut duration = Duration::days(serial as i64); 
    let frac = (serial - serial.floor()) as i64; 
    if frac > 0 {
        duration = duration.checked_add(&Duration::hours(24 * frac)).expect("Could not add two dates together"); 
    }
    start_date.checked_add_signed(duration).unwrap()
}

pub fn column_number_to_letter(col_idx: usize) -> String {
    let alpha = String::from("ABCDEFGHIJKLMNOPQRSTUVWXYZ");
    let mut col_name: Vec<char> = vec![];
    let mut n = col_idx;
    while n > 0 {
        let rem: usize = n % 26;
        if rem == 0 {
            col_name.push('Z');
            n = (n / 26) - 1;
        } else {
            col_name.push(alpha.chars().nth(rem - 1).unwrap());
            n /= 26;
        }
    }
    col_name.into_iter().rev().collect::<String>()
}


//pub fn adjust_formula(
    //base_reference: Reference,
    //current_reference: Reference,
    //formula_text: String,
//) -> Result<String, Error> {
    //let row_offset: i32 = current_reference.row() as i32 - base_reference.row() as i32;
    //let column_offset: i32 = current_reference.column() as i32 - base_reference.column() as i32;
    //let mut expression: Expr = parse_str(&formula_text)?; 
    //adjust_expression(row_offset, column_offset, &mut expression)?; 
	//Ok(format!("{}", expression))
//}

//pub fn adjust_expression(
    //row_offset: i32, 
    //column_offset: i32, 
    //expression: &mut Expr
//) -> Result<(), Error> {
    //match *expression {
        //Expr::Reference { sheet: _, ref mut reference } => {
            //let mut r = Reference::from(reference.to_string());
			//r.offset((row_offset, column_offset));
			//*reference = r.to_string(); 
        //}, 
        //Expr::Infix(_, ref mut a, ref mut b) => {
            //adjust_expression(row_offset, column_offset, a)?; 
            //adjust_expression(row_offset, column_offset, b)?; 
        //}, 
        //Expr::Prefix(_, ref mut a) => {
            //adjust_expression(row_offset, column_offset, a)?; 
        //}, 
        //Expr::Func { name: _, ref mut args } => {
            //for arg in args.iter_mut() {
                //adjust_expression(row_offset, column_offset, arg)?; 
            //} 
        //}, 
        //Expr::Array(ref mut arr) => {
            //for a in arr.iter_mut() {
                //adjust_expression(row_offset, column_offset, a)?; 
            //}
        //}
        //_ => {}
    //}
    //Ok(())
//}

//#[cfg(test)]
//mod tests {
    ////use crate::reference::Reference; 
    //use crate::utils::adjust_formula; 
    //use crate::errors::Error; 
    
    //#[test]
    //fn test_adjust_formula() -> Result<(), Error> {
       //let base_reference = Reference::from((1, 1));
       //let current_reference = Reference::from((2, 1));
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("Sheet1!A1"))?, &"Sheet1!A2"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("A1+A2"))?, &"(A2+A3)"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("SUM(A1+A2)"))?, &"SUM((A2+A3))"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("{A1, A2}"))?, &"{A2, A3}"); 
        //Ok(())
    //}

    //#[test]
    //fn test_adjust_formula_anchored() -> Result<(), Error> {
       //let base_reference = Reference::from((1, 1));
       //let current_reference = Reference::from((2, 2));
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("Sheet1!$A$1"))?, &"Sheet1!$A$1"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("$A$1+A2"))?, &"($A$1+B3)"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("SUM(A$1+A$2)"))?, &"SUM((B$1+B$2))"); 
       //assert_eq!(&adjust_formula(base_reference, current_reference, String::from("{A1, A2}"))?, &"{B2, B3}"); 
        //Ok(())
    //}
//}
