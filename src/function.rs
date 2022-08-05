use crate::parse::{Expr, Error}; 
use crate::evaluate::Value; 
use from_args_derive::FromArgs;

pub trait Function {
   fn evaluate(self) -> Result<Value, Error>; 
}

#[derive(FromArgs)]
pub struct Sum {
    pub a: Value,  
    pub b: Value 
}

impl Function for Sum {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num() + self.b.as_num()))
        } else {
            Err(Error::Value)
        }
    }
}

