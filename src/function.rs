use crate::evaluate::value::Value; 
use function_macro::function; 

pub fn get_function_value(name: &str, args: Vec<Value>) -> Value {
    match name {
		"SUM" => Box::new(Sum::from(args)).evaluate(), 
		"AVERAGE" => Box::new(Average::from(args)).evaluate(), 
		"COUNT" => Box::new(Count::from(args)).evaluate(),	
		"EXPONENT" => Box::new(Exponent::from(args)).evaluate(),	
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
                println!("{:?}", x); 
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

#[cfg(test)]
mod tests {
    use crate::evaluate::evaluate_str;
    use crate::evaluate::value::Value; 

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
 
}
