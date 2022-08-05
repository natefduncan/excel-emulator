use crate::parse::{Expr, Error}; 
use crate::evaluate::Value; 
use from_args_derive::FromArgs;

pub trait Function {
   fn evaluate(self) -> Result<Value, Error>; 
}

#[derive(FromArgs)]
pub struct Add {
    pub a: Value,  
    pub b: Value 
}

impl Function for Add {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num() + self.b.as_num()))
        } else {
            Err(Error::Value)
        }
    }
}

#[derive(FromArgs)]
pub struct Subtract {
    pub a: Value, 
    pub b: Value,
}

impl Function for Subtract {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num() - self.b.as_num()))
        } else {
            Err(Error::Value)
        }
    }
}

#[derive(FromArgs)]
pub struct Multiply {
    pub a: Value,
    pub b: Value, 
}

impl Function for Multiply {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num() * self.b.as_num()))
        } else {
            Err(Error::Value)
        }
    }
}

#[derive(FromArgs)]
pub struct Divide {
    pub a: Value,
    pub b: Value, 
}

impl Function for Divide {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num() / self.b.as_num()))
        } else {
            Err(Error::Value)
        }
    }
}

#[derive(FromArgs)]
pub struct Exponent {
    pub a: Value,
    pub b: Value, 
}

impl Function for Exponent {
    fn evaluate(self) -> Result<Value, Error> {
        if self.a.is_num() && self.b.is_num() {
            Ok(Value::from(self.a.as_num().powf(self.b.as_num())))
        } else {
            Err(Error::Value)
        }
    }
}

