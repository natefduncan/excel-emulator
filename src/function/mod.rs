pub mod xirr; 

use crate::{
    evaluate::{
        value::Value, 
    }, 
    reference::Reference, 
    cell::Cell, 
}; 
use function_macro::function; 
use chrono::{Months, naive::NaiveDate, Datelike}; 

pub fn get_function_value(name: &str, args: Vec<Value>) -> Value {
    match name {
		"SUM" => Box::new(Sum::from(args)).evaluate(), 
		"AVERAGE" => Box::new(Average::from(args)).evaluate(), 
		"COUNT" => Box::new(Count::from(args)).evaluate(),	
		"EXPONENT" => Box::new(Exponent::from(args)).evaluate(),	
		"CONCAT" => Box::new(Concat::from(args)).evaluate(),	
		"AND" => Box::new(Andfunc::from(args)).evaluate(),	
		"OR" => Box::new(Orfunc::from(args)).evaluate(),	
		"MAX" => Box::new(Max::from(args)).evaluate(),	
		"MIN" => Box::new(Min::from(args)).evaluate(),	
		"MATCH" => Box::new(Matchfn::from(args)).evaluate(),	
		"INDEX" => Box::new(Index::from(args)).evaluate(),	
		"DATE" => Box::new(Date::from(args)).evaluate(),	
		"FLOOR" => Box::new(Floor::from(args)).evaluate(),	
		"IFERROR" => Box::new(Iferror::from(args)).evaluate(),	
		"EOMONTH" => Box::new(Eomonth::from(args)).evaluate(),	
		"SUMIFS" => Box::new(Sumifs::from(args)).evaluate(),	
		"XIRR" => Box::new(Xirrfunc::from(args)).evaluate(),	
		"IF" => Box::new(Iffunc::from(args)).evaluate(),	
		"XNPV" => Box::new(Xnpv::from(args)).evaluate(),	
        _ => panic!("Function {} does not convert to a value.", name)  
    }
}

pub trait Function {
   fn evaluate(self) -> Value; 
}

pub fn offset(r: &mut Reference, rows: i32, cols: i32, height: Option<usize>, width: Option<usize>) -> Reference {
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
            _ => panic!("Match statement could not resolve.")
        }
    } else if match_type.as_num() == 0.0 {
        match lookup_array_mut.into_iter().position(|v| v == lookup_value) {
            Some(v) => { Value::from(v + 1) }, 
            _ => panic!("Match statement could not resolve.")
        }
    } else {
        // Largest value that is less than or equal to the lookup-value
        // Lookup array placed in ascending order.
        lookup_array_mut.sort(); // Ascending Order
        match lookup_array_mut.into_iter().enumerate().filter(|(_, v)| v <= &lookup_value).last() {
            Some(v) => { Value::from(v.0 + 1) }, 
            _ => panic!("Match statement could not resolve.")
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

#[function]
fn index(arr: Value, row_num: Value, col_num: Value) -> Value {
    arr.as_array2()[[row_num.as_num() as usize - 1, col_num.as_num() as usize - 1]].clone()
}

#[function]
fn iferror(a: Value, b: Value) -> Value {
    if a.is_err() {
        b 
    } else {
        a
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
        let criteria: &Value = args.get(i+1).unwrap(); 
        for (i, cell) in cell_range.into_iter().enumerate() {
            if &cell == criteria && !keep_index.contains(&i) {
                keep_index.push(i); 
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
    Value::from(xirr::compute(&payments).unwrap())
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
    println!("rate: {:?}", rate); 
    let rate: f64 = rate.as_num(); 
    let values: Vec<f64> = values.as_array().iter().map(|x| x.as_num()).collect(); 
    let dates: Vec<NaiveDate> = dates.as_array().iter().map(|x| x.as_date()).collect(); 
    let start_date = dates.get(0).unwrap().clone(); 
    println!("rate2: {:?}", rate); 
    Value::from(
        values
        .into_iter()
        .zip(
            dates
            .into_iter()
        ).fold(0.0, |s, (value, date)| {
            let days = NaiveDate::signed_duration_since(date, start_date).num_days() as f64; 
            println!("{:?}", days); 
            s + (value / ((1.0+rate).powf(days / 365.0)))
        })
    ) 
}


#[cfg(test)]
mod tests {
    use crate::{
        evaluate::{
            value:: Value, 
            evaluate_str 
        },
        workbook::Book
    };
    use chrono::naive::NaiveDate; 

	#[test]
    fn test_sum() {
		assert_eq!(evaluate_str("SUM(1,2,3,4,5)"), Value::from(15.0));
		assert_eq!(evaluate_str("SUM({1,2;3,4})"), Value::from(10.0));
		assert_eq!(evaluate_str("SUM({1,2,3,4,5},6,\"7\")"), Value::from(28.0));
		assert_eq!(evaluate_str("SUM({1,\"2\",TRUE,4})"), Value::from(5.0));
    }

    #[test]
    fn test_average() {
		assert_eq!(evaluate_str("AVERAGE(1,2,3,4,5)"), Value::from(3.0));
		assert_eq!(evaluate_str("AVERAGE({1,2;3,4})"), Value::from(2.5));
		assert_eq!(evaluate_str("AVERAGE({1,2,3,4,5},6,\"7\")"), Value::from(4.0));
		assert_eq!(evaluate_str("AVERAGE({1,\"2\",TRUE,4})"), Value::from(2.5));
    }

    #[test]
    fn test_count() {
		assert_eq!(evaluate_str("COUNT(1,2,3,4,5)"), Value::from(5.0));
		assert_eq!(evaluate_str("COUNT({1,2,3,4,5})"), Value::from(5.0));
		assert_eq!(evaluate_str("COUNT({1,2,3,4,5},6,\"7\")"), Value::from(7.0));
    }
 
    #[test]
    fn test_concat() {
		assert_eq!(evaluate_str("CONCAT(\"test\", \"func\")"), Value::from("testfunc".to_string()));
    }

    #[test]
    fn test_and() {
		assert_eq!(evaluate_str("AND(TRUE, TRUE)"), Value::from(true));
    }

    #[test]
    fn test_or() {
		assert_eq!(evaluate_str("OR(TRUE, FALSE)"), Value::from(true));
    }

    #[test]
    fn test_max_min() {
		assert_eq!(evaluate_str("MAX(1, 5, 10)"), Value::from(10.0));
		assert_eq!(evaluate_str("MIN(1, 5, 10)"), Value::from(1.0));
    }

    #[test]
    fn test_match() {
		assert_eq!(evaluate_str("MATCH(3, {1,2,3,4,5}, 0)"), Value::from(3.0));
    }

    #[test]
    fn test_index() {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate(); 
        assert_eq!(book.resolve_str_ref("Sheet1!H3")[[0,0]].as_num(), 11.0); 
    }

    #[test]
    fn test_date() {
		assert_eq!(evaluate_str("DATE(2022, 1, 1)"), Value::from(NaiveDate::from_ymd(2022, 1, 1)));
    }

    #[test]
    fn test_floor() {
        assert_eq!(evaluate_str("FLOOR(3.7, 1)"), Value::from(3.0)); 
        // assert_eq!(evaluate_str("FLOOR(-2.5, -2)"), Value::from(-2.0)); 
        // assert_eq!(evaluate_str("FLOOR(1.58, 0.01)"), Value::from(1.5)); 
        // assert_eq!(evaluate_str("FLOOR(0.234, 0.01)"), Value::from(0.23)); 
    }

    #[test]
    fn test_iferror() {
        assert_eq!(evaluate_str("IFERROR(#VALUE!, 1)"), Value::from(1.0)); 
    }

    #[test]
    fn test_eomonth() {
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 2, 29), 12)"), Value::from(NaiveDate::from_ymd(2005, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 2, 28), 12)"), Value::from(NaiveDate::from_ymd(2005, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 1, 15), -23)"), Value::from(NaiveDate::from_ymd(2002, 2, 28))); 
        assert_eq!(evaluate_str("EOMONTH(DATE(2004, 1, 15), 0)"), Value::from(NaiveDate::from_ymd(2004, 1, 31))); 
    }

    #[test]
    fn test_sumifs() {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate(); 
        assert_eq!(book.resolve_str_ref("Sheet1!H5")[[0,0]].as_num(), 2.0); 
    }

    #[test]
    fn test_xirr() {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate(); 
        assert!((0.3340 - book.resolve_str_ref("Sheet1!H4")[[0,0]].as_num()).abs() < 0.01); 
    }

    #[test]
    fn test_offset() {
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate(); 
        assert_eq!(book.resolve_str_ref("Sheet1!H6")[[0,0]].as_num(), 10.0); 
    }
    
    #[test]
    fn test_if() {
        assert_eq!(evaluate_str("IF(TRUE, 1, 2)"), Value::from(1.0)); 
        assert_eq!(evaluate_str("IF(FALSE, 1, 2)"), Value::from(2.0)); 
    }

    #[test]
    fn test_xnpv() {
        assert_eq!(evaluate_str("IF(TRUE, 1, 2)"), Value::from(1.0)); 
        let mut book = Book::from("assets/functions.xlsx"); 
        book.load().unwrap(); 
        book.calculate(); 
        println!("{:?}", book.resolve_str_ref("Sheet1!H7")); 
        assert!((7.657 - book.resolve_str_ref("Sheet1!H7")[[0,0]].as_num()).abs() < 0.01); 
    }
 


}
