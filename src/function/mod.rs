pub mod xirr; 

use crate::{
    evaluate::{
        evaluate_expr_with_context, 
        evaluate_str, 
        ensure_non_range,
        value::Value, 
    }, 
    reference::Reference, 
    cell::Cell, 
    errors::Error, 
    parser::ast::{Expr, Error as ExcelError},  
    workbook::Book,
}; 
use function_macro::function; 
use chrono::{Months, naive::NaiveDate, Datelike}; 

pub fn get_function_value(name: &str, args: Vec<Value>) -> Result<Value, Error> {
    match name {
		"SUM" => Ok(Box::new(Sum::from(args)).evaluate()), 
		"AVERAGE" => Ok(Box::new(Average::from(args)).evaluate()), 
		"COUNT" => Ok(Box::new(Count::from(args)).evaluate()),	
		"EXPONENT" => Ok(Box::new(Exponent::from(args)).evaluate()),	
		"CONCAT" => Ok(Box::new(Concat::from(args)).evaluate()),	
		"AND" => Ok(Box::new(Andfunc::from(args)).evaluate()),	
		"OR" => Ok(Box::new(Orfunc::from(args)).evaluate()),	
		"MAX" => Ok(Box::new(Max::from(args)).evaluate()),	
		"MIN" => Ok(Box::new(Min::from(args)).evaluate()),	
		"MATCH" => Ok(Box::new(Matchfn::from(args)).evaluate()),	
		"DATE" => Ok(Box::new(Date::from(args)).evaluate()),	
		"FLOOR" => Ok(Box::new(Floor::from(args)).evaluate()),	
		"IFERROR" => {
            let a = args.get(0).unwrap().clone(); 
            let b = args.get(1).unwrap().clone(); 
            Ok(Box::new(Iferror { a, b }).evaluate())
        },	
		"EOMONTH" => Ok(Box::new(Eomonth::from(args)).evaluate()),	
		"SUMIFS" => Ok(Box::new(Sumifs::from(args)).evaluate()),	
		"XIRR" => Ok(Box::new(Xirrfunc::from(args)).evaluate()),	
		"IF" => Ok(Box::new(Iffunc::from(args)).evaluate()),	
		"XNPV" => Ok(Box::new(Xnpv::from(args)).evaluate()),	
		"YEARFRAC" => Ok(Box::new(Yearfrac::from(args)).evaluate()),	
		"DATEDIF" => Ok(Box::new(Datedif::from(args)).evaluate()),	
		"PMT" => Ok(Box::new(Pmt::from(args)).evaluate()),	
		"COUNTA" => Ok(Box::new(Counta::from(args)).evaluate()),	
		"ROUNDDOWN" => Ok(Box::new(Rounddown::from(args)).evaluate()),	
        _ => Err(Error::FunctionNotSupport(name.to_string()))
    }
}

pub trait Function {
   fn evaluate(self) -> Value; 
}

pub fn offset_reference(r: &mut Reference, rows: i32, cols: i32, height: Option<usize>, width: Option<usize>) -> Reference {
    if r.row() as i32 + rows < 0 || r.column() as i32 + cols < 0 {
        panic!("Invalid offset");
    } else {
        r.offset((rows, cols));
    }
    let mut end_cell : Option<Cell> = None;  
    if height.is_some() || width.is_some() {
        let h_u : usize = height.unwrap_or(0); 
        let w_u : usize = width.unwrap_or(0); 
        if h_u > 1 || w_u > 1 {
            end_cell = Some(
                Cell::from((
                    r.row() + h_u, 
                    r.column() + w_u
                ))
            ); 
        }
    }
    r.end_cell = end_cell; 
    *r
}

#[function]
fn exponent(a: Value, b: Value) -> Value {
    Value::from(a.as_num().powf(b.as_num()))
}

#[function]
fn sum(args: Vec<Value>) -> Value {
    args.into_iter().fold(Value::from(0.0), |mut s, v| {
        if let Value::Array(arr) = v {
            for x in arr {
                if x.is_num() {
                    s += x
                }
            }
        } else if let Value::Array2(arr2) = v {
            for x in arr2 {
                if x.is_num() {
                    s += x
                }
            }
        } else {
            s += Value::from(v.as_num())
        }
        s
    })
}

#[function]
fn average(args: Vec<Value>) -> Value {
    let mut count = 0.0;
    let mut sum_values: Vec<Value> = vec![]; 
    for arg in args.into_iter() {
        if let Value::Array(arr) = arg {
            for x in arr {
                if x.is_num() {
                    sum_values.push(x); 
                    count += 1.0; 
                }
            }
        } else {
            sum_values.push(Value::from(arg.as_num()));
            count += 1.0; 
        }
   }
    let average = sum_values.into_iter().fold(0.0, |mut s, v| {
        s += v.as_num();
        s
    }) / count;
    Value::from(average)
}

#[function]
fn count(args: Vec<Value>) -> Value {
	let mut count = 0.0;
	for arg in args.iter() {
		if let Value::Array(arr) = arg {
            for x in arr.iter() {
                if x.is_num() {
                    count += 1.0; 
                }
            }
        } else {
            count += 1.0; 
        }
	}
    Value::from(count)
}

#[function]
fn concat(a: Value, b: Value) -> Value {
    Value::from(format!("{}{}", a.as_text(), b.as_text()))
}

#[function]
fn andfunc(a: Value, b: Value) -> Value {
    Value::from(a.as_bool() && b.as_bool())
}

#[function]
fn orfunc(a: Value, b: Value) -> Value {
    Value::from(a.as_bool() || b.as_bool())
}

#[function]
fn max(args: Vec<Value>) -> Value {
    let mut output = args[0].clone(); 
    for v in args.into_iter() {
        if let Value::Array(arr) = v {
            for x in arr {
                if x.is_num() {
                    output = output.max(x); 
                }
            }
        } else if let Value::Array2(arr2) = v {
            for x in arr2 {
                if x.is_num() {
                    output = output.max(x); 
                }
            }
        } else {
            output = output.max(v); 
        }
    }
    output
}

#[function]
fn min(args: Vec<Value>) -> Value {
    let mut output = args[0].clone(); 
    for v in args.into_iter() {
        if let Value::Array(arr) = v {
            for x in arr {
                if x.is_num() {
                    output = output.min(x); 
                }
            }
        } else if let Value::Array2(arr2) = v {
            for x in arr2 {
                if x.is_num() {
                    output = output.min(x); 
                }
            }
        } else {
            output = output.min(v); 
        }
    }
    output
}

#[function]
fn matchfn(lookup_value: Value, lookup_array: Value, match_type: Value) -> Value {
    let mut lookup_array_mut = lookup_array.as_array();
    if match_type.as_num() == -1.0 {
        // Smallest value that is greater than or equal to the lookup-value.
        // Lookup array placed in descending order.
        lookup_array_mut.sort_by(|a, b| b.cmp(a)); // Descending Order
        match lookup_array.as_array().into_iter().enumerate().filter(|(_,v)| v >= &lookup_value).last() {
            Some(v) => { Value::from(v.0 + 1) },
            _ => Value::Error(ExcelError::NA)
        }
    } else if match_type.as_num() == 0.0 {
        match lookup_array_mut.into_iter().position(|v| v == lookup_value) {
            Some(v) => { Value::from(v + 1) }, 
            _ => Value::Error(ExcelError::NA)
        }
    } else {
        // Largest value that is less than or equal to the lookup-value
        // Lookup array placed in ascending order.
        lookup_array_mut.sort(); // Ascending Order
        match lookup_array_mut.into_iter().enumerate().filter(|(_, v)| v <= &lookup_value).last() {
            Some(v) => { Value::from(v.0 + 1) }, 
            _ => Value::Error(ExcelError::NA)
        }
    }
}

#[function]
fn date(year: Value, month: Value, day: Value) -> Value {
   Value::from(NaiveDate::from_ymd(year.as_num() as i32, month.as_num() as u32, day.as_num() as u32))
}


#[function]
// FIXME: significance
fn floor(x: Value, _significance: Value) -> Value {
    Value::from(math::round::floor(x.as_num(), 0))
}

/*
 * Index function can return either a value or a reference. 
 * Excel treats them different depending on what the parent function needs.
 * This function will always return a Value::Ref and require than 
 * conversion to an actual value happens higher up the evaluation chain. 
*/
pub fn index(args: Vec<Expr>, book: &Book) -> Result<Value, Error> {
	let mut arg_values = args.into_iter(); 
	let array: Value = evaluate_expr_with_context(arg_values.next().unwrap(), book)?; // This can be a range or an array
	let row_num: Value = evaluate_expr_with_context(arg_values.next().unwrap(), book)?; 
	let col_num_option = arg_values.next(); 
	let col_num = match col_num_option {
		Some(expr) => evaluate_expr_with_context(expr, book)?,
		None => Value::from(1.0)
	}; 
    let row_idx = row_num.as_num() as usize - 1;
    let col_idx = col_num.as_num() as usize - 1; 
    if let Value::Range { sheet, reference, value } = array {
		let reference = Reference::from(reference); 
		let (start_row, start_col, _, _) = reference.get_dimensions(); 

        // If row value is zero, reference entire column.
        // Start cell row index is zero. 
		if row_num.as_num() == 0.0 {
            let new_col = start_col + col_idx; 
			return Ok(Value::Range { sheet: sheet.clone(), reference: Reference::from((0, new_col)), value: None }); 
		}

        // If column value is zero, reference entire column.
        // Start cell column index is zero. 
		if col_num.as_num() == 0.0 {
            let new_row = start_row + row_idx; 
			return Ok(Value::Range { sheet: sheet.clone(), reference: Reference::from((new_row, 0)), value: None }); 
		}

        let new_row = start_row + row_idx;  
        let new_col = start_col + col_idx; 
        let new_value: Value = value.unwrap().as_array2()[[row_idx, col_idx]].clone(); 
        return Ok(Value::Range { sheet: sheet.clone(), reference: Reference::from((new_row, new_col)), value: Some(Box::new(new_value)) }); 
	} else {
		panic!("First argument must be a range."); 
	}
} 

pub fn offset(args: Vec<Expr>, book: &Book) -> Result<Value, Error> {
    let array = evaluate_expr_with_context(args.get(0).unwrap().clone(), book)?; 
	if let Value::Range { sheet, reference, value: _ } = array { 
		let rows = ensure_non_range(evaluate_expr_with_context(args.get(1).unwrap().clone(), book)?);
		let cols = ensure_non_range(evaluate_expr_with_context(args.get(2).unwrap().clone(), book)?); 
		let height = args.get(3); 
		let height_opt: Option<usize> = height.map(|h| {
			ensure_non_range(evaluate_expr_with_context(h.clone(), book).unwrap()).as_num() as usize
		}); 
		let width = args.get(4); 
		let width_opt: Option<usize> = width.map(|w| {
			ensure_non_range(evaluate_expr_with_context(w.clone(), book).unwrap()).as_num() as usize
		}); 
		let new_reference = offset_reference(&mut reference.clone(), rows.as_num() as i32, cols.as_num() as i32, height_opt, width_opt); 
        let new_expr = Expr::Reference { sheet: sheet.clone(), reference: new_reference.to_string() }; 
        if book.is_calculated(new_expr.clone()) {
            let reference_value = match evaluate_expr_with_context(new_expr.clone(), book) {
                Ok(value) => Some(Box::new(ensure_non_range(value))), 
                _ => panic!("New expression could not be evaluated: {}", new_expr.clone())
            }; 
            Ok(Value::Range { sheet: sheet.clone(), reference: new_reference, value:  reference_value})
        } else {
            Err(Error::Volatile(Box::new(new_expr)))
        }
    } else {
        panic!("First expression must be a Reference.")
    }
}

struct Iferror {
    a: Value, 
    b: Value, 
}

impl Function for Iferror {
    fn evaluate(self) -> Value {
        if self.a.is_err() {
            self.b 
        } else {
            self.a
        }
    }
}

#[function]
fn eomonth(start_date: Value, months: Value) -> Value {
    let start_date: NaiveDate = start_date.as_date(); 
    let bom = NaiveDate::from_ymd(start_date.year(), start_date.month(), 1);
    let eom: NaiveDate; 
    if months.as_num() > 0.0 {
        eom = bom.checked_add_months(Months::new((months.as_num()+1.0) as u32)).unwrap(); 
    } else if months.as_num() < 0.0 {
        eom = bom.checked_sub_months(Months::new((months.as_num()*-1.0-1.0) as u32)).unwrap(); 
    } else {
        eom = bom.checked_add_months(Months::new(1)).unwrap(); 
    }
    Value::from(eom.pred())
}

#[function]
// TODO: Beef up criteria support
fn sumifs(sum_range: Value, args: Vec<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    for i in (0..args.len()).step_by(2) {
        let cell_range: Vec<Value> = args.get(i).unwrap().as_array(); 
        let criteria: Value = args.get(i+1).unwrap().ensure_single(); 
        let criteria_text = format!("{}", criteria); 
        for (i, cell) in cell_range.into_iter().enumerate() {
            let cell_text = format!("{}", cell.ensure_single()); 
            let eval = if criteria_text.contains("<") || criteria_text.contains(">") {
                evaluate_str(format!("{}{}", cell_text, criteria_text).as_str()).unwrap()
            } else {
                evaluate_str(format!("{}={}", cell_text, criteria_text).as_str()).unwrap()
            }; 
            if let Value::Bool(x) = eval {
                if x && !keep_index.contains(&i) {
                    keep_index.push(i); 
                }
            }
       }
    } 
    Value::from(sum_range.as_array()
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| match keep_index.contains(&i) {
            true => Some(v.as_num()), 
            false => None
        }) 
        .collect::<Vec<f64>>()
        .iter()
        .sum::<f64>()) 
} 

#[function]
fn xirrfunc(values: Value, dates: Value) -> Value {
    let payments: Vec<xirr::Payment> = values
        .as_array()
        .iter()
        .zip(
            dates
            .as_array()
            .iter()
        ).map(|(v, d)| xirr::Payment { amount: v.as_num(), date: d.as_date() })
        .collect(); 
    match xirr::compute(&payments) {
        Ok(v) => Value::from(v), 
        _ => Value::Error(ExcelError::Num), 
    }
}

#[function]
fn iffunc(condition: Value, a: Value, b: Value) -> Value {
    if condition.as_bool() {
        a
    } else {
        b
    }
}

#[function]
fn xnpv(rate: Value, values: Value, dates: Value) -> Value {
    let rate: f64 = rate.as_num(); 
    let dates: Vec<NaiveDate> = dates.as_array().iter().map(|x| x.as_date()).collect(); 
    let start_date = *dates.get(0).unwrap(); 
    Value::from(
        values.as_array().iter().map(|x| x.as_num())
        .into_iter()
        .zip(
            dates
            .into_iter()
        ).fold(0.0, |s, (value, date)| {
            let days = NaiveDate::signed_duration_since(date, start_date).num_days() as f64; 
            s + (value / ((1.0+rate).powf(days / 365.0)))
        })
    ) 
}

#[function]
//TODO: Implement basis
fn yearfrac(start_date: Value, end_date: Value) -> Value {
    let start_date: NaiveDate = start_date.as_date(); 
    let end_date: NaiveDate = end_date.as_date(); 
    Value::from(
        (
            ((end_date.year() as i32 - start_date.year() as i32) * 360) + 
            (end_date.ordinal() as i32 - start_date.ordinal() as i32)
        ) as f64 / 360.0
    )    
}

#[function]
fn datedif(start_date: Value, end_date: Value, unit: Value) -> Value {
    let start_date: NaiveDate = start_date.as_date(); 
    let end_date: NaiveDate = end_date.as_date(); 
    match unit.as_text().as_str() {
        "Y" | "y" => Value::from(end_date.year() - start_date.year()),
        "M" | "m" => Value::from((end_date.year() as i32 - start_date.year() as i32)*12 + (end_date.month() as i32 - start_date.month() as i32)),
        "D" | "d" => Value::from(NaiveDate::signed_duration_since(end_date, start_date).num_days() as f64),
        "MD" | "md" => Value::from(end_date.day() as i32 - start_date.day() as i32), 
        "YM" | "ym" => Value::from(end_date.month() as i32 - start_date.month() as i32), 
        "YD" | "yd" => Value::from(end_date.ordinal() as i32 - start_date.ordinal() as i32),
        _ => panic!("Not a valid unit.")
    }
}

#[function]
fn pmt(rate: Value, nper: Value, pv: Value, fv: Option<Value>, f_type: Option<Value>) -> Value {
    let rate = rate.as_num();
    let nper = nper.as_num();
    let pv = pv.as_num();
    let fv = fv.unwrap_or_else(|| Value::from(0.0)).as_num(); 
    let f_type = f_type.unwrap_or_else(|| Value::from(0.0)).as_num();
    Value::from(
        rate*(fv*-1.0+pv*(1.0+rate).powf(nper))/((1.0+rate*f_type)*(1.0-(1.0+rate).powf(nper)))
    )
}

#[function]
fn counta(args: Vec<Value>) -> Value {
    Value::from(
        args.into_iter().fold(0, |s, v| {
            match v {
                Value::Array(arr) => {
                    s + arr.into_iter().fold(0, |s, v| match v {
                        Value::Empty => s, 
                            _ => s + 1
                    })
                },
                Value::Array2(arr2) => {
                    s + arr2.into_raw_vec().into_iter().fold(0, |s, v| match v {
                        Value::Empty => s, 
                        _ => s + 1
                    })
                }, 
                _ => s + 1
            }
        })
    )
}

#[function]
fn rounddown(x: Value, num_digits: Value) -> Value {
    let x: f64 = x.as_num(); 
    let num_digits: f64 = num_digits.as_num(); 
    if num_digits > 0.0 {
        Value::from(((x * 10.0_f64.powf(num_digits)) as i64) as f64 / 10.0_f64.powf(num_digits))
    } else if num_digits < 0.0 {
        Value::from(((x / 10.0_f64.powf(-num_digits)) as i64) as f64 * 10.0_f64.powf(-num_digits))
    } else {
        Value::from((x as i64) as f64)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        evaluate::{
            value:: Value, 
            evaluate_str 
        },
        workbook::Book,
        errors::Error, 
    };
    use chrono::naive::NaiveDate; 

    #[test]
    fn test_rounddown() -> Result<(), Error> {
        assert_eq!(evaluate_str("ROUNDDOWN(3.2, 0)")?, Value::from(3.0)); 
        assert_eq!(evaluate_str("ROUNDDOWN(76.9, 0)")?, Value::from(76.0)); 
        assert_eq!(evaluate_str("ROUNDDOWN(3.14159, 3)")?, Value::from(3.141)); 
        assert_eq!(evaluate_str("ROUNDDOWN(-3.14159, 1)")?, Value::from(-3.1)); 
        assert_eq!(evaluate_str("ROUNDDOWN(31415.92654, -2)")?, Value::from(31400)); 
        Ok(())
    }

    #[test]
    fn test_counta() -> Result<(), Error> {
        assert_eq!(evaluate_str("COUNTA(1,2,3,4,5)")?, Value::from(5.0)); 
        assert_eq!(evaluate_str("COUNTA({1,2,3,4,5})")?, Value::from(5.0)); 
        assert_eq!(evaluate_str("COUNTA({1,2,3,4,5},6,\"7\")")?, Value::from(7.0)); 
        Ok(())
    }

    #[test]
    fn test_pmt() -> Result<(), Error> {
        assert!((-1037.03 - evaluate_str("PMT(0.08/12, 10, 10000)")?.as_num()).abs() < 0.01); 
        assert!((-1030.16 - evaluate_str("PMT(0.08/12, 10, 10000, 0, 1)")?.as_num()).abs() < 0.01); 
        Ok(())
    }

    #[test]
    fn test_datedif() -> Result<(), Error> {
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"Y\")")?, Value::from(16.0)); 
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"M\")")?, Value::from(193.0)); 
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"D\")")?, Value::from(5873.0)); 
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"YM\")")?, Value::from(1.0)); 
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"MD\")")?, Value::from(0.0)); 
        assert_eq!(evaluate_str("DATEDIF(DATE(2004, 2, 10), DATE(2020, 3, 10), \"YD\")")?, Value::from(29.0)); 
        Ok(())
    }

	#[test]
    fn test_sum() -> Result<(), Error> {
		assert_eq!(evaluate_str("SUM(1,2,3,4,5)")?, Value::from(15.0));
		assert_eq!(evaluate_str("SUM({1,2;3,4})")?, Value::from(10.0));
		assert_eq!(evaluate_str("SUM({1,2,3,4,5},6,\"7\")")?, Value::from(28.0));
		assert_eq!(evaluate_str("SUM({1,\"2\",TRUE,4})")?, Value::from(5.0));
        Ok(())
    }

    #[test]
    fn test_average() -> Result<(), Error> {
		assert_eq!(evaluate_str("AVERAGE(1,2,3,4,5)")?, Value::from(3.0));
		assert_eq!(evaluate_str("AVERAGE({1,2;3,4})")?, Value::from(2.5));
		assert_eq!(evaluate_str("AVERAGE({1,2,3,4,5},6,\"7\")")?, Value::from(4.0));
		assert_eq!(evaluate_str("AVERAGE({1,\"2\",TRUE,4})")?, Value::from(2.5));
        Ok(())
    }

    #[test]
    fn test_count() -> Result<(), Error> {
		assert_eq!(evaluate_str("COUNT(1,2,3,4,5)")?, Value::from(5.0));
		assert_eq!(evaluate_str("COUNT({1,2,3,4,5})")?, Value::from(5.0));
		assert_eq!(evaluate_str("COUNT({1,2,3,4,5},6,\"7\")")?, Value::from(7.0));
        Ok(())
    }
 
    #[test]
    fn test_concat() -> Result<(), Error> {
		assert_eq!(evaluate_str("CONCAT(\"test\", \"func\")")?, Value::from("testfunc".to_string()));
        Ok(())
    }

    #[test]
    fn test_and() -> Result<(), Error> {
		assert_eq!(evaluate_str("AND(TRUE, TRUE)")?, Value::from(true));
        Ok(())
    }

    #[test]
    fn test_or() -> Result<(), Error>  {
		assert_eq!(evaluate_str("OR(TRUE, FALSE)")?, Value::from(true));
        Ok(())
    }

    #[test]
    fn test_max_min() -> Result<(), Error> {
		assert_eq!(evaluate_str("MAX(1, 5, 10)")?, Value::from(10.0));
		assert_eq!(evaluate_str("MIN(1, 5, 10)")?, Value::from(1.0));
        Ok(())
    }

    #[test]
    fn test_match() -> Result<(), Error> {
		assert_eq!(evaluate_str("MATCH(3, {1,2,3,4,5}, 0)")?, Value::from(3.0));
        Ok(())
    }

    #[test]
    fn test_index() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate()?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H3")?[[0,0]].as_num(), 11.0); 
        Ok(())
    }

    #[test]
    fn test_date() -> Result<(), Error> {
		assert_eq!(evaluate_str("DATE(2022, 1, 1)")?, Value::from(NaiveDate::from_ymd(2022, 1, 1)));
        Ok(())
    }

    #[test]
    fn test_floor() -> Result<(), Error> {
        assert_eq!(evaluate_str("FLOOR(3.7, 1)")?, Value::from(3.0)); 
        // assert_eq!(evaluate_str("FLOOR(-2.5, -2)"), Value::from(-2.0)); 
        // assert_eq!(evaluate_str("FLOOR(1.58, 0.01)"), Value::from(1.5)); 
        // assert_eq!(evaluate_str("FLOOR(0.234, 0.01)"), Value::from(0.23)); 
        Ok(())
    }

    #[test]
    fn test_iferror() -> Result<(), Error> {
        assert_eq!(evaluate_str("IFERROR(#VALUE!, 1)")?, Value::from(1.0)); 
        Ok(())
    }

    #[test]
    fn test_eomonth() -> Result<(), Error> {
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 2, 29), 12)")?, Value::from(NaiveDate::from_ymd(2005, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 2, 28), 12)")?, Value::from(NaiveDate::from_ymd(2005, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 1, 15), -23)")?, Value::from(NaiveDate::from_ymd(2002, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 1, 15), 0)")?, Value::from(NaiveDate::from_ymd(2004, 1, 31))); 
        Ok(())
    }

    #[test]
    fn test_sumifs() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate()?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H5")?[[0,0]].as_num(), 2.0); 
        Ok(())
    }

    #[test]
    fn test_xirr() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate()?; 
        assert!((0.3340 - book.resolve_str_ref("Sheet1!H4")?[[0,0]].as_num()).abs() < 0.01); 
        Ok(())
    }

    #[test]
    fn test_offset() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate()?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H6")?[[0,0]].as_num(), 10.0); 
        Ok(())
    }
    
    #[test]
    fn test_if() -> Result<(), Error> {
        assert_eq!(evaluate_str("IF(TRUE, 1, 2)")?, Value::from(1.0)); 
        assert_eq!(evaluate_str("IF(FALSE, 1, 2)")?, Value::from(2.0)); 
        Ok(())
    }

    #[test]
    fn test_xnpv() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate()?; 
        assert!((7.657 - book.resolve_str_ref("Sheet1!H7")?[[0,0]].as_num()).abs() < 0.01); 
        Ok(())
    }

    #[test]
    fn test_yearfrac() -> Result<(), Error> {
        assert!((0.58055 - evaluate_str("YEARFRAC(DATE(2012, 1, 1), DATE(2012, 7, 30))")?.as_num() < 0.01)); 
        Ok(())
    }
}
