use crate::parse::Expr; 
use crate::evaluate::Value; 
use function_macro::excel_function; 

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

