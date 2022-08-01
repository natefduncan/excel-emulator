use time::Date; 
use std::fmt; 

use crate::parse::{Expr, Opcode, Error}; 
use crate::function::*; 

#[derive(Clone)]
pub enum Value { 
    Num(f32), 
    Bool(bool), 
    Text(String), 
    Date(Date), 
    Array(Vec<Value>), 
}

impl From<f32> for Value { fn from(f: f32) -> Value { Value::Num(f) }}
impl From<bool> for Value { fn from(b: bool) -> Value { Value::Bool(b) }}
impl From<String> for Value { fn from(s: String) -> Value { Value::Text(s) }}
impl From<&str> for Value { fn from(s: &str) -> Value { Value::Text(s.to_string()) }}
impl From<Vec<Value>> for Value { fn from(v: Vec<Value>) -> Value { Value::Array(v) }}
impl From<Date> for Value { fn from(d: Date) -> Value { Value::Date(d) }}
impl From<Expr> for Value {
    fn from(expr: Expr) -> Value {
        match expr {
            Expr::Num(x) => Value::from(x),
            Expr::Bool(x) => Value::from(x), 
            Expr::Text(x) => Value::from(x.clone()),
            Expr::Array(x) => Value::from(x.to_vec()), 
            Expr::Op(a, opcode, b) => {
                match opcode {
                    Opcode::Add => Sum {a: Value::from(*a), b: Value::from(*b)}.evaluate().unwrap(), 
                    _ => Value::from(-1.0) // TODO
                }
            }, 
            Expr::Func {name, args } => {
                match name.as_str() {
                    "SUM" => Sum::from(args).evaluate().unwrap(), 
                    _ => Value::from(-1.0)
                }
            }, 
            _ => Value::from(-1.0) // TODO
        }
    }
}
impl From<Vec<Box<Expr>>> for Value {
    fn from(v: Vec<Box<Expr>>) -> Value {
       Value::from(v.to_vec().into_iter().map(|x| Value::from(*x)).collect::<Vec<Value>>())
    }
}
impl From<Box<Expr>> for Value {
    fn from(bexpr: Box<Expr>) -> Value {
        Value::from(*bexpr)
    }
}


impl Value {
    pub fn is_num(&self) -> bool { matches!(self, Value::Num(_)) }
    pub fn is_bool(&self) -> bool { matches!(self, Value::Bool(_)) }
    pub fn is_text(&self) -> bool { matches!(self, Value::Text(_)) }
    pub fn is_date(&self) -> bool { matches!(self, Value::Date(_)) }
    pub fn is_array(&self) -> bool { matches!(self, Value::Array(_)) }

    pub fn as_num(&self) -> f32 {
        if let Value::Num(x) = self {
            *x
        } else {
            panic!("{} cannot be converted to a number.", self); 
        }
    }

    pub fn as_bool(&self) -> bool {
        if let Value::Bool(x) = self {
            *x
        } else {
            panic!("{} cannot be converted to a boolean.", self); 
        }
    }

    pub fn as_text(&self) -> String {
        if let Value::Text(x) = self {
            x.clone()
        } else {
            panic!("{} cannot be converted to a string.", self); 
        }
    }

    pub fn as_date(&self) -> Date {
        if let Value::Date(x) = self {
            *x
        } else {
            panic!("{} cannot be converted to a date.", self); 
        }
    }

    pub fn as_array(&self) -> Vec<Value> {
        if let Value::Array(x) = self {
            x.to_vec()
        } else {
            panic!("{} cannot be converted to an array.", self); 
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Value::Num(x) => { write!(f, "{}", x) }, 
            Value::Bool(x) => { write!(f, "{}", x) }, 
            Value::Text(x) => { write!(f, "{}", x) }, 
            Value::Date(x) => { write!(f, "{}", x) }, 
            Value::Array(x) => {
                x.iter().fold(Ok(()), |result, output| {
                    result.and_then(|_| writeln!(f, "{}", output)) 
                })
            }, 
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::evaluate::*;
    use crate::excel::ExprParser; 

    fn evaluate_expr(expr_str: &str) -> String {
        println!("{}", expr_str); 
        let expr : Box<Expr> = ExprParser::new().parse(expr_str).unwrap(); 
        println!("{:?}", expr); 
        format!("{}", Value::from(expr)) 
    }

    #[test]
    fn test_num() {
        assert_eq!(&evaluate_expr(" 1 + 1 "), "2");
    }

    #[test]
    fn test_formula() {
        assert_eq!(&evaluate_expr(" SUM(1, 1) "), "2"); 
    }
}
