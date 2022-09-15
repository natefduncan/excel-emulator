use crate::evaluate::value::Value; 
use function_macro::function; 

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
        _ => panic!("Function {} does not convert to a value.", name)  
    }
}

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
fn index(arr: Value, row_num: Value, col_num: Value) -> Value {
    arr.as_array2()[[row_num.as_num() as usize - 1, col_num.as_num() as usize - 1]].clone()
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

}
