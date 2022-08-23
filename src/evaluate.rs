use chrono::NaiveDate; 
use std::fmt; 
use std::cmp::{Eq, PartialEq, PartialOrd, Ordering};
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign};  

use crate::parse::{Expr, Opcode}; 
use crate::function::*; 

type NumType = f32;
type BoolType = bool;
type TextType = String; 
type ArrayType = Vec<Value>;
type DateType = NaiveDate; 

#[derive(Clone, PartialEq, Debug)]
pub enum Value { 
    Num(NumType), 
    Bool(BoolType), 
    Text(TextType), 
    Date(DateType), 
    Array(ArrayType), 
    Formula(TextType), 
    Empty
}

impl From<f32> for Value { fn from(f: NumType) -> Value { Value::Num(f) }}
impl From<bool> for Value { fn from(b: BoolType) -> Value { Value::Bool(b) }}
impl From<String> for Value { fn from(s: TextType) -> Value { Value::Text(s) }}
impl From<&str> for Value { fn from(s: &str) -> Value { Value::Text(s.to_string()) }}
impl From<Vec<Value>> for Value { fn from(v: ArrayType) -> Value { Value::Array(v) }}
impl From<NaiveDate> for Value { fn from(d: DateType) -> Value { Value::Date(d) }}
impl From<Expr> for Value {
    fn from(expr: Expr) -> Value {
        match expr {
            Expr::Num(x) => Value::from(x),
            Expr::Bool(x) => Value::from(x), 
            Expr::Text(x) => Value::from(x),
            Expr::Array(x) => Value::from(x.to_vec()), 
            Expr::Op(a, opcode, b) => {
                match opcode {
                    Opcode::Add => Value::from(*a) + Value::from(*b), 
                    Opcode::Subtract => Value::from(*a) - Value::from(*b), 
                    Opcode::Multiply => Value::from(*a) * Value::from(*b), 
                    Opcode::Divide => Value::from(*a) / Value::from(*b), 
                    Opcode::Exponent => Exponent {a: Value::from(*a), b: Value::from(*b)}.evaluate(), 
                    Opcode::Equal => Value::from(Value::from(a) == Value::from(b)), 
                    Opcode::NotEqual => Value::from(Value::from(a) != Value::from(b)), 
                    Opcode::LessThan => Value::from(Value::from(a) < Value::from(b)), 
                    Opcode::LessThanOrEqual => Value::from(Value::from(a) <= Value::from(b)), 
                    Opcode::GreaterThan => Value::from(Value::from(a) > Value::from(b)), 
                    Opcode::GreaterThanOrEqual => Value::from(Value::from(a) >= Value::from(b)), 
                    _ => panic!("Opcode {} does not convert to a value.", opcode) 
                }
            }, 
            Expr::Func {name, args} => evaluate_function(name.as_str(), args),
            _ => panic!("Expression {} does not convert to a value.", expr)  
        }
    }
}
impl From<Vec<Box<Expr>>> for Value {
    fn from(v: Vec<Box<Expr>>) -> Value {
       Value::from(v.iter().cloned().map(|x| Value::from(*x)).collect::<Vec<Value>>())
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
    pub fn is_empty(&self) -> bool { matches!(self, Value::Empty) }
    pub fn is_formula(&self) -> bool { matches!(self, Value::Formula(_)) }

    pub fn as_num(&self) -> NumType {
        match self {
            Value::Num(x) => *x, 
            Value::Text(t) => t.parse::<NumType>().unwrap(), 
            Value::Bool(x) => {
                match x {
                    true => 1.0, 
                    false => 0.0
                }
            }, 
            _ => panic!("{} cannot be converted to a number.", self)
        }
    }

    pub fn as_bool(&self) -> BoolType {
        match self {
            Value::Bool(x) => *x, 
            Value::Num(n) => {
                if *n == 1.0 {
                    true
                } else if *n == 0.0 {
                    false
                } else {
                    panic!("{} cannot be converted to a boolean.", self)
                }
            }, 
            _ => panic!("{} cannot be converted to a boolean.", self)
        }
    }

    pub fn as_text(&self) -> TextType {
        match self {
            Value::Text(x) 
            | Value::Formula(x) => x.clone(), 
            _ => panic!("{} cannot be converted to a string.", self)
        } 
   }

    pub fn as_date(&self) -> DateType {
        if let Value::Date(x) = self {
            *x
        } else {
            panic!("{} cannot be converted to a date.", self); 
        }
    }

    pub fn as_array(&self) -> ArrayType {
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
            Value::Bool(x) => { write!(f, "{}", if *x { "TRUE" } else { "FALSE" }) }, 
            Value::Text(x) | Value::Formula(x) => { write!(f, "{}", x) }, 
            Value::Date(x) => { write!(f, "{}", x) }, 
            Value::Array(x) => {
                x.iter().fold(Ok(()), |result, output| {
                    result.and_then(|_| writeln!(f, "{}", output)) 
                })
            }, 
            Value::Empty => { write!(f, "Empty") }
        }
    }
}

impl Eq for Value { }

fn variant_ord(v : &Value) -> usize {
    let variants : Vec<bool> = vec![
        v.is_bool(),
        v.is_text(),
        v.is_num(),
        v.is_date()
    ];
    let variant_len : usize = variants.len();
    match variants.into_iter().position(|x| x) {
        Some(u) => {
            u
        },
        None => {
            variant_len
        }
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        let self_rank : usize = variant_ord(self);
        let other_rank : usize = variant_ord(other);
        match self_rank.cmp(&other_rank) {
            Ordering::Greater => {
                Some(Ordering::Greater)
            },
            Ordering::Less => {
                Some(Ordering::Less)
            },
            Ordering::Equal => {
                if self.is_bool() {
                    Some(self.as_bool().cmp(&other.as_bool()))
                } else if self.is_text() {
                    Some(self.as_text().cmp(&other.as_text()))
                } else if self.is_num() {
                    self.as_num().partial_cmp(&other.as_num())
                } else if self.is_date() {
                    Some(self.as_date().cmp(&other.as_date()))
                } else {
                    None
                }
            }
        }
    }
}

impl Add for Value {
    type Output = Self; 
    fn add(self, other: Self) -> Self {
           match self {
               Value::Num(x) => Value::from(x + other.as_num()), 
               Value::Text(ref x) => Value::from(format!("{}{}", x, other.as_text())),
               Value::Bool(_) => Value::from(self.as_num() + other.as_num()), 
               //TODO
               _ => panic!("{} cannot be added to {}.", other, self)
           }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        if self.is_num() {
            *self = self.clone() + other
        } else {
            panic!("{} cannot be add assigned to {}.", other, self)
        }
    }
}

impl Sub for Value {
    type Output = Self; 
    fn sub(self, other: Self) -> Self {
           match self {
               Value::Num(x) => Value::from(x - other.as_num()), 
               Value::Bool(_) => Value::from(self.as_num() - other.as_num()), 
               // TODO
               _ => panic!("{} cannot be subtracted from {}.", other, self)
           }
    }
}

impl Mul for Value {
    type Output = Self; 
    fn mul(self, other: Self) -> Self {
           match self {
               Value::Num(x) => Value::from(x * other.as_num()), 
               Value::Bool(_) => Value::from(self.as_num() * other.as_num()), 
               // TODO
               _ => panic!("{} cannot be multiplied by {}.", self, other)
           }
    }
}

impl Div for Value {
    type Output = Self; 
    fn div(self, other: Self) -> Self {
           match self {
               Value::Num(x) => Value::from(x / other.as_num()), 
               // TODO
               _ => panic!("{} cannot be multiplied by {}.", self, other)
           }
    }
}

impl Neg for Value {
    type Output = Self;
    fn neg(self) -> Self {
        Value::from(-self.as_num())
    }
}


#[cfg(test)]
mod tests {
	use crate::utils::evaluate_expr;

    #[test]
    fn test_op_codes() {
        assert_eq!(&evaluate_expr(" 1 + 1 "), "2");
        assert_eq!(&evaluate_expr(" 1 - 1 "), "0"); 
        assert_eq!(&evaluate_expr(" 2 * 2 "), "4"); 
        assert_eq!(&evaluate_expr(" (2 + 1) * 2 "), "6"); 
        assert_eq!(&evaluate_expr(" 8 / 4 "), "2"); 
        assert_eq!(&evaluate_expr(" 8^2 "), "64"); 
    }

    #[test]
    fn test_conditionals() {
        assert_eq!(&evaluate_expr(" 1=1 "), "TRUE"); 
    }

    #[test]
    fn test_formula() {
        assert_eq!(&evaluate_expr(" SUM(1, 1) "), "2"); 
        assert_eq!(&evaluate_expr(" SUM(SUM(1, 2), 1) "), "4"); 
    }
}
