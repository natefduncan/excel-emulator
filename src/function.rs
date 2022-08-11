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
    args.into_iter().fold(Value::from(0.0), |mut s, v| {s += v; s})
}

