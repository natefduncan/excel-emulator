use chrono::{Duration, NaiveDate}; 
use std::fmt; 
use std::cmp::{Eq, PartialEq, PartialOrd, Ordering};
use std::ops::{Add, Sub, Mul, Div, Neg, AddAssign};  
use ndarray::Array2; 

use crate::reference::Reference;
use crate::parser::ast::Error; 

type NumType = f64;
type BoolType = bool;
type TextType = String; 
type ArrayType = Vec<Value>;
type Array2Type = Array2<Value>;
type DateType = NaiveDate; 
type ErrorType = Error; 

#[derive(Clone, PartialEq, Debug)]
pub enum Value { 
    Num(NumType), 
    Bool(BoolType), 
    Text(TextType), 
    Date(DateType), 
    Array(ArrayType), 
    Array2(Array2Type), 
    Formula(TextType), 
    Error(ErrorType), 
    Range { sheet: Option<String>, reference: Reference, value: Option<Box<Value>> }, 
    Empty
}

impl From<f64> for Value { fn from(f: NumType) -> Value { Value::Num(f) }}
impl From<i32> for Value { fn from(f: i32) -> Value { Value::Num(f as f64) }}
impl From<usize> for Value { fn from(f: usize) -> Value { Value::Num(f as f64) }}
impl From<bool> for Value { fn from(b: BoolType) -> Value { Value::Bool(b) }}
impl From<String> for Value { fn from(s: TextType) -> Value { Value::Text(s) }}
impl From<&str> for Value { fn from(s: &str) -> Value { Value::Text(s.to_string()) }}
impl From<Vec<Value>> for Value { fn from(v: ArrayType) -> Value { Value::Array(v) }}
impl From<Array2<Value>> for Value { fn from(v: Array2Type) -> Value { Value::Array2(v) }}
impl From<NaiveDate> for Value { fn from(d: DateType) -> Value { Value::Date(d) }}

impl Value {
    pub fn is_num(&self) -> bool { matches!(self, Value::Num(_)) }
    pub fn is_bool(&self) -> bool { matches!(self, Value::Bool(_)) }
    pub fn is_text(&self) -> bool { matches!(self, Value::Text(_)) }
    pub fn is_date(&self) -> bool { matches!(self, Value::Date(_)) }
    pub fn is_array(&self) -> bool { matches!(self, Value::Array(_)) }
    pub fn is_array2(&self) -> bool { matches!(self, Value::Array2(_)) }
    pub fn is_empty(&self) -> bool { matches!(self, Value::Empty) }
    pub fn is_formula(&self) -> bool { matches!(self, Value::Formula(_)) }
    pub fn is_range(&self) -> bool { matches!(self, Value::Range {sheet: _, reference: _, value: _}) }
    pub fn is_err(&self) -> bool { matches!(self, Value::Error(_)) }

    pub fn ensure_single(&self) -> Value {
        match self {
            Value::Array2(arr2) => arr2[[0,0]].ensure_single().clone(), // assume single
            Value::Array(arr) => arr.get(0).unwrap().ensure_single().clone(), // assume single
            c => c.clone() // TODO
        }
    }

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
            Value::Array2(arr2) => { // Assume single cell
                arr2[[0,0]].as_num()
            }, 
            Value::Empty => 0.0, 
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
            Value::Array2(arr2) => { // Assume single cell
                arr2[[0,0]].as_text()
            }, 
            _ => panic!("{} cannot be converted to a string.", self)
        } 
    }

    pub fn as_date(&self) -> DateType {
        match self { 
            Value::Date(x) => *x,
            Value::Array2(arr2) => {
                arr2[[0,0]].as_date()
            }, 
            _ => panic!("{} cannot be converted to a date.", self)
        }
    }

    pub fn as_array(&self) -> ArrayType {
        match self {
            Value::Array(arr) => arr.to_vec(),
            Value::Array2(arr2) => arr2.clone().into_raw_vec(), 
            _ => panic!("{} cannot be converted to an array.", self)
        }
    }

    pub fn as_array2(&self) -> Array2Type {
        match self {
            Value::Array2(arr2) => arr2.clone(), 
            _ => panic!("{} cannot be converted to an array2.", self)
        }
    }

    pub fn as_err(&self) -> ErrorType {
        if let Value::Error(err) = self {
            err.clone()
        } else {
            panic!("{} cannot be converted to an error.", self)
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
            Value::Range {sheet, reference, value: _} => { 
                match sheet {
                    Some(s) => write!(f, "{}!{}", s, reference), 
                    None => write!(f, "{}", reference)
                }
            }, 
            Value::Array2(arr2) => write!(f, "{}", arr2), 
            Value::Error(err) => write!(f, "{}", err)
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
                    let a = self.as_num();
                    let b = other.as_num();
                    if a > b {
                        Some(Ordering::Greater)
                    } else if a < b {
                        Some(Ordering::Less)
                    } else {
                        Some(Ordering::Equal)
                    }
                } else if self.is_date() {
                    Some(self.as_date().cmp(&other.as_date()))
                } else {
                    None
                }
            }
        }
    }
}

impl Ord for Value {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Add for Value {
    type Output = Self; 
    fn add(self, other: Self) -> Self {
        match self.ensure_single() {
            Value::Num(x) => Value::from(x + other.ensure_single().as_num()), 
            Value::Text(ref x) => Value::from(format!("{}{}", x, other.ensure_single().as_text())),
            Value::Bool(_) => Value::from(self.as_num() + other.ensure_single().as_num()), 
            Value::Empty => Value::from(0.0 + other.ensure_single().as_num()), 
            Value::Date(dt) => {
                Value::from(dt.checked_add_signed(Duration::days(other.ensure_single().as_num() as i64)).unwrap())
            }, 
            _ => panic!("{} cannot be added to {}.", other, self)
        }
    }
}

impl AddAssign for Value {
    fn add_assign(&mut self, other: Self) {
        if self.ensure_single().is_num() {
            *self = self.ensure_single() + other
        } else {
            panic!("{} cannot be add assigned to {}.", other, self)
        }
    }
}

impl Sub for Value {
    type Output = Self; 
    fn sub(self, other: Self) -> Self {
        match self.ensure_single() {
            Value::Num(x) => Value::from(x - other.ensure_single().as_num()), 
            Value::Bool(_) => Value::from(self.as_num() - other.ensure_single().as_num()), 
            Value::Empty => Value::from(0.0 - other.ensure_single().as_num()), 
            Value::Date(dt) => {
                let other_single = other.ensure_single(); 
                if other_single.is_date() {
                        Value::from(NaiveDate::signed_duration_since(dt, other_single.as_date()).num_days() as f64)
                } else {
                    Value::from(dt.checked_sub_signed(Duration::days(other_single.as_num() as i64)).unwrap())
                }
            }, 
            _ => panic!("{} cannot be subtracted from {}.", other, self)
        }
    }
}

impl Mul for Value {
    type Output = Self; 
    fn mul(self, other: Self) -> Self {
        match self.ensure_single() {
            Value::Num(x) => Value::from(x * other.ensure_single().as_num()), 
            Value::Bool(_) => Value::from(self.as_num() * other.ensure_single().as_num()), 
            Value::Empty => Value::from(0.0 * other.ensure_single().as_num()), 
            // TODO
            _ => panic!("{} cannot be multiplied by {}.", self, other)
        }
    }
}

impl Div for Value {
    type Output = Self; 
    fn div(self, other: Self) -> Self {
        match self.ensure_single() {
            Value::Num(x) => Value::from(x / other.ensure_single().as_num()), 
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
