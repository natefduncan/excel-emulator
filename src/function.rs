use crate::parse::{Expr, Error}; 
use crate::evaluate::Value; 

pub trait Function {
   fn evaluate(self) -> Result<Value, Error>; 
}

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

impl From<Vec<Box<Expr>>> for Sum {
    fn from(mut v: Vec<Box<Expr>>) -> Sum {
        let b = Value::from(*v.pop().unwrap());
        let a = Value::from(*v.pop().unwrap());
        Sum { a, b }
    }
}

// Macro that generates a From trait for <T>. 
// Convert vector args to named, value equivalents. 
// Create struct <T> and evaluate. 
// Going from Expr::Function(name, args) to Value. 
