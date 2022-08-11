use crate::parse::Expr; 
use crate::evaluate::Value; 
use function_macro::excel_function; 

pub fn evaluate_function(name: &str, args: Vec<Box<Expr>>) -> Value {
	match name {
		"SUM" => Sum::from(args).evaluate(), 
		"AVERAGE" => Average::from(args).evaluate(), 
		_ => panic!("Function {} does not convert to a value.", name)  
	}
}

pub trait Function {
   fn evaluate(self) -> Value; 
}

#[excel_function]
fn exponent(a: Value, b: Value) -> Value {
    Value::from(a.as_num().powf(b.as_num()))
}

#[excel_function]
fn sum(args: Vec<Value>) -> Value {
    args.into_iter().fold(Value::from(0.0), |mut s, v| {
        if let Value::Array(arr) = v {
            for x in arr {
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

#[excel_function]
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

#[cfg(test)]
mod tests {
    use crate::utils::evaluate_expr;

	#[test]
    fn test_sum() {
		assert_eq!(&evaluate_expr("SUM(1,2,3,4,5)"), "15");
		assert_eq!(&evaluate_expr("SUM({1,2;3,4})"), "10");
		assert_eq!(&evaluate_expr("SUM({1,2,3,4,5},6,\"7\")"), "28");
		assert_eq!(&evaluate_expr("SUM({1,\"2\",TRUE,4})"), "5");
    }

    #[test]
    fn test_average() {
		assert_eq!(&evaluate_expr("AVERAGE(1,2,3,4,5)"), "3");
		assert_eq!(&evaluate_expr("AVERAGE({1,2;3,4})"), "2.5");
		assert_eq!(&evaluate_expr("AVERAGE({1,2,3,4,5},6,\"7\")"), "4");
		assert_eq!(&evaluate_expr("AVERAGE({1,\"2\",TRUE,4})"), "2.5");
    }
 
}
