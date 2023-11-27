pub mod xirr; 

use crate::{
    evaluate::{
        evaluate_str, 
        value::Value, 
    }, 
    parser::ast::{Expr, Error as ExcelError},  
}; 
use excel_emulator_macro::function; 
use chrono::{Months, naive::NaiveDate, Datelike}; 

pub trait Function {
   fn evaluate(self) -> Value; 
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
    let lookup_value = lookup_value.ensure_single(); 
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

pub struct Iferror {
    pub a: Value, 
    pub b: Value, 
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
fn sumifs(sum_range: Value, args: Vec<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    for (idx, i) in (0..args.len()).step_by(2).enumerate() {
        let cell_range: Vec<Value> = args.get(i).unwrap().as_array();
        let criteria: Value = args.get(i+1).unwrap().ensure_single(); 
        let criteria_text = criteria.as_text(); 
        for (y, cell) in cell_range.iter().enumerate() {
            let eval: bool = parse_criteria(criteria_text.as_str(), cell); 
            if idx == 0 {
                if eval {
                    keep_index.push(y); 
                }
            } else {
                if ! eval && keep_index.contains(&y) {
                    keep_index.retain(|x| x != &y); 
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
fn countifs(args: Vec<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    for (idx, i) in (0..args.len()).step_by(2).enumerate() {
        let cell_range: Vec<Value> = args.get(i).unwrap().as_array();
        let criteria: Value = args.get(i+1).unwrap().ensure_single(); 
        let criteria_text = criteria.as_text(); 
        for (y, cell) in cell_range.iter().enumerate() {
            let eval: bool = parse_criteria(criteria_text.as_str(), cell); 
            if idx == 0 {
                if eval {
                    keep_index.push(y); 
                }
            } else {
                if ! eval && keep_index.contains(&y) {
                    keep_index.retain(|x| x != &y); 
                }
           }
       } 
    }
    Value::from(keep_index.len())
} 


#[function]
fn sumif(range: Value, criteria: Value, sum_range: Option<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    let range: Vec<Value> = range.as_array(); 
    let criteria = criteria.ensure_single(); 
    let criteria_text = format!("{}", criteria); 
    for (i, cell) in range.iter().enumerate() {
        let eval = parse_criteria(criteria_text.as_str(), cell); 
        if eval && !keep_index.contains(&i) {
            keep_index.push(i); 
        }
    } 
    let sum_range = match sum_range {
        Some(val) => val.as_array(), 
        None => range
    }; 
    Value::from(sum_range
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

fn parse_criteria(c: &str, cell: &Value) -> bool {
    let cell = cell.ensure_single().as_text(); 
    let mut op: &str = if c.split("<>").count() > 1 {
        "<>"
    } else if c.split("<=").count() > 1 {
        "<="
    } else if c.split("<").count() > 1 {
        "<"
    } else if c.split(">=").count() > 1 {
        ">="
    } else if c.split(">").count() > 1 {
        ">"
    } else if c.split("=").count() > 1 {
        "="
    } else {
        "" 
    }; 
    let lh: String; 
    let rh: String; 
    if ! op.is_empty() {
        lh = c.split(op).collect::<Vec<&str>>()[1].replace("\"", "").to_string(); 
        rh = cell.replace("\"", ""); 
    } else {
        lh = c.replace("\"", "").to_string(); 
        rh = cell.replace("\"", ""); 
        op = "="; 
    } 
    evaluate_str(format!("\"{}\"{}\"{}\"", lh, op, rh).as_str()).unwrap().as_bool()
}

#[function]
fn averageif(range: Value, criteria: Value, average_range: Option<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    let range: Vec<Value> = range.as_array(); 
    let criteria = criteria.ensure_single(); 
    let criteria_text = criteria.as_text(); 
    for (i, cell) in range.iter().enumerate() {
        let eval = parse_criteria(criteria_text.as_str(), cell); 
        if eval && !keep_index.contains(&i) {
            keep_index.push(i); 
        }
    } 
    let average_range = match average_range {
        Some(val) => val.as_array(), 
        None => range
    }; 
    let average_range_filter = average_range
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| match keep_index.contains(&i) {
            true => Some(v.as_num()), 
            false => None
        }).collect::<Vec<f64>>(); 
    Value::from(average_range_filter
        .iter()
        .sum::<f64>()/average_range_filter.len() as f64)
} 



#[function]
fn averageifs(average_range: Value, args: Vec<Value>) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    for i in (0..args.len()).step_by(2) {
        let cell_range: Vec<Value> = args.get(i).unwrap().as_array(); 
        let criteria: Value = args.get(i+1).unwrap().ensure_single(); 
        let criteria_text = criteria.as_text(); 
        for (i, cell) in cell_range.iter().enumerate() {
            let eval = parse_criteria(criteria_text.as_str(), cell); 
            if eval && !keep_index.contains(&i) {
                keep_index.push(i); 
            }
        } 
    } 
    let average_range_filter = average_range.as_array()
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| match keep_index.contains(&i) {
            true => Some(v.as_num()), 
            false => None
        }).collect::<Vec<f64>>(); 
    Value::from(average_range_filter
        .iter()
        .sum::<f64>()/average_range_filter.len() as f64) 
} 

#[function]
fn sumproduct(args: Vec<Value>) -> Value {
    let args: Vec<Vec<Value>> = args.into_iter().map(|x| x.as_array()).collect(); 
    let mut output = Value::from(0.0); 
    for i in 0..args[0].len() {
        let mut a = Value::from(1.0); 
        for j in 0..args.len() {
            a = a * args[j][i].clone(); 
        }
        output += a; 
    }
    output
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
    let value = rate*(fv*-1.0+pv*(1.0+rate).powf(nper))/((1.0+rate*f_type)*(1.0-(1.0+rate).powf(nper)));
    if value == f64::INFINITY || value == f64::NEG_INFINITY {
        Value::Error(ExcelError::Num)
    } else {
        Value::from(value)
    }
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

//FIXME
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

//FIXME
#[function]
fn roundup(x: Value, num_digits: Value) -> Value {
    let x: f64 = x.as_num(); 
    let num_digits: f64 = num_digits.as_num(); 
    if num_digits > 0.0 {
        Value::from((((x * 10.0_f64.powf(num_digits)) as i64 + x.signum() as i64) as f64) / 10.0_f64.powf(num_digits))
    } else if num_digits < 0.0 {
        Value::from((((x / 10.0_f64.powf(-num_digits)) as i64 + x.signum() as i64) as f64) * 10.0_f64.powf(-num_digits))
    } else {
        Value::from((x as i64 + x.signum() as i64) as f64)
    }
}


// TODO: Wildcard usage
#[function]
fn search(find_text: Value, within_text: Value, start_num: Option<Value>) -> Value {
    let find_text = find_text.as_text().to_lowercase(); 
    let within_text = within_text.as_text().to_lowercase(); 
    let start_num = start_num.unwrap_or(Value::from(1.0)).as_num() as usize - 1; 
    let mut within_text_chars = within_text.chars(); 
    for _ in 0..start_num {
        within_text_chars.next(); 
    }
    if let Some(idx) =  (&within_text_chars.collect::<String>()).find(&find_text) {
        Value::from(idx + start_num + 1)
    } else {
        Value::Error(ExcelError::Value)
    }
}
 
#[function]
fn countif(range: Value, criteria: Value) -> Value {
    let mut keep_index: Vec<usize> = vec![]; 
    let range: Vec<Value> = range.as_array(); 
    let criteria = criteria.ensure_single(); 
    let criteria_text = format!("{}", criteria); 
    for (i, cell) in range.iter().enumerate() {
        let eval = parse_criteria(criteria_text.as_str(), cell); 
        if eval && !keep_index.contains(&i) {
            keep_index.push(i); 
        }
    } 
    Value::from(range
        .into_iter()
        .enumerate()
        .filter_map(|(i, v)| match keep_index.contains(&i) {
            true => Some(v.as_num()), 
            false => None
        }) 
        .collect::<Vec<f64>>()
        .iter()
        .count())
} 

#[function]
fn month(date: Value) -> Value {
    Value::from(date.as_date().month() as f64)
}

#[function]
fn year(date: Value) -> Value {
    Value::from(date.as_date().year() as f64)
}


#[cfg(test)]
mod tests {
    use crate::{
        evaluate::{
            value:: Value, 
            evaluate_str 
        },
        errors::Error, 
    };
    use chrono::naive::NaiveDate; 

    #[test]
    fn test_sumproduct() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H9")?[[0,0]].as_num(), 530.0); 
        Ok(())
    }

    #[test]
    fn test_search() -> Result<(), Error> {
        assert_eq!(evaluate_str("SEARCH(\"a\",\"Apple\") ")?, Value::from(1.0)); 
        assert_eq!(evaluate_str("SEARCH(\"the\",\"The cat in the hat\")")?, Value::from(1.0)); 
        assert_eq!(evaluate_str("SEARCH(\"the\",\"The cat in the hat\",4)")?, Value::from(12.0)); 
        Ok(())
    }

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
    fn test_roundup() -> Result<(), Error> {
        assert_eq!(evaluate_str("ROUNDUP(3.2, 0)")?, Value::from(4.0)); 
        assert_eq!(evaluate_str("ROUNDUP(76.9, 0)")?, Value::from(77.0)); 
        assert_eq!(evaluate_str("ROUNDUP(3.14159, 3)")?, Value::from(3.142)); 
        assert_eq!(evaluate_str("ROUNDUP(-3.14159, 1)")?, Value::from(-3.2)); 
        assert_eq!(evaluate_str("ROUNDUP(31415.92654, -2)")?, Value::from(31500)); 
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
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
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
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H5")?[[0,0]].as_num(), 2.0); 
        Ok(())
    }

    #[test]
    fn test_averageifs() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
        assert_eq!(book.resolve_str_ref("Sheet1!H8")?[[0,0]].as_num(), 2.0); 
        Ok(())
    }

    #[test]
    fn test_xirr() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
        assert!((0.3340 - book.resolve_str_ref("Sheet1!H4")?[[0,0]].as_num()).abs() < 0.01); 
        Ok(())
    }

    #[test]
    fn test_offset() -> Result<(), Error> {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
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
        book.load(false).unwrap(); 
        book.calculate(false, false)?; 
        assert!((7.657 - book.resolve_str_ref("Sheet1!H7")?[[0,0]].as_num()).abs() < 0.01); 
        Ok(())
    }

    #[test]
    fn test_yearfrac() -> Result<(), Error> {
        assert!((0.58055 - evaluate_str("YEARFRAC(DATE(2012, 1, 1), DATE(2012, 7, 30))")?.as_num() < 0.01)); 
        Ok(())
    }
}
