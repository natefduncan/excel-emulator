use crate::{
    parser::{
        ast::{Literal, Prefix, Infix, Expr}, 
        parse_str
    }, 
    function::*, 
    workbook::Book,  
    evaluate::value::Value, 
    reference::Reference, 
    errors::Error, 
}; 

pub mod value; 

pub fn evaluate_str(s: &str) -> Result<Value, Error> {
    let expr = parse_str(s)?; 
    evaluate_expr(expr)
}

pub fn evaluate_expr(expr: Expr) -> Result<Value, Error> {
     let value = match expr {
        Expr::Func {name, args} => {
            let arg_values: Vec<Value> = args.into_iter().map(|x| evaluate_expr(x).unwrap()).collect::<Vec<Value>>(); 
            get_function_value(&name, arg_values)?
        }, 
        Expr::Literal(lit) => {
			match lit {
				Literal::Number(f) => {
                    if f.is_nan() {
                        Value::from(0.0)
                    } else {
                        Value::from(f)
                    }
                }
				Literal::Boolean(b) => Value::from(b), 
				Literal::Text(s) => Value::from(s)
			}
		},
		Expr::Prefix(p, box_expr) => { 
			match p {
				Prefix::Plus => Value::from(evaluate_expr(*box_expr)?.as_num().abs()),
				Prefix::Minus => evaluate_expr(*box_expr)? * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
            let a = evaluate_expr(*a)?; 
            let b = evaluate_expr(*b)?; 
            if a.is_err() {
                return Ok(a); 
            } else if b.is_err() {
                return Ok(b); 
            } else {
                match i {
                    Infix::Plus => a + b, 
                    Infix::Minus => a - b, 
                    Infix::Multiply => a * b, 
                    Infix::Divide => a / b, 
                    Infix::Exponent => Exponent {a, b}.evaluate(), 
                    Infix::NotEqual => Value::from(a != b), 
                    Infix::Equal => Value::from(a == b), 
                    Infix::LessThan => Value::from(a < b), 
                    Infix::LessThanEqual => Value::from(a <= b), 
                    Infix::GreaterThan => Value::from(a > b), 
                    Infix::GreaterThanEqual => Value::from(a >= b), 
                    Infix::Ampersand => {
                        let value = if a.is_array() {
                            Value::from(a.as_array().into_iter().map(|x| Value::from(format!("{}{}", x.as_text(), b.as_text()))).collect::<Vec<Value>>())
                        } else if b.is_array() {
                            Value::from(b.as_array().into_iter().map(|x| Value::from(format!("{}{}", a.as_text(), x.as_text()))).collect::<Vec<Value>>())
                        } else {
                            Value::from(format!("{}{}", a.as_text(), b.as_text()))
                        }; 
                        value
                    },
                }
            }
        }, 
        Expr::Array(x) => Value::Array(x.into_iter().map(|x| evaluate_expr(x).unwrap()).collect::<Vec<Value>>()), 
        Expr::Error(err) => Value::Error(err), 
        _ => panic!("Expression {:?} does not convert to a value.", expr)  
	}; 
    Ok(value)
}

pub fn offset_expr(args: Vec<Expr>, book: &Book, debug: bool) -> Result<Expr, Error> {
    if let Expr::Reference { sheet, reference } = args.get(0).unwrap() { 
        let rows = evaluate_expr_with_context(args.get(1).unwrap().clone(), book, debug)?;
        let cols = evaluate_expr_with_context(args.get(2).unwrap().clone(), book, debug)?; 
        let height = args.get(3); 
        let height_opt: Option<i32> = height.map(|h| {
            evaluate_expr_with_context(h.clone(), book, debug).unwrap().as_num() as i32 
        }); 
        let width = args.get(4); 
        let width_opt: Option<i32> = width.map(|w| {
            evaluate_expr_with_context(w.clone(), book, debug).unwrap().as_num() as i32 
        }); 
        let new_reference = offset_reference(&mut Reference::from(reference.as_str()), rows.as_num() as i32, cols.as_num() as i32, height_opt, width_opt); 
        Ok(Expr::Reference { sheet: sheet.clone(), reference: new_reference.to_string() })
    } else {
        panic!("Offset must have a reference.")
    }
}

pub fn ensure_non_range(value: Value) -> Value {
    if let Value::Range { sheet: _, reference: _, value } = value {
        if let Some(value) = value {
            return *value; 
        } else {
            panic!("Value::Range is missing a value to return")
        }
    } else {
        return value; 
    }
}

pub fn evaluate_expr_with_context(expr: Expr, book: &Book, debug: bool) -> Result<Value, Error> {
    let value = match expr.clone() {
        Expr::Reference { ref sheet, ref reference } => {
            let range_value: Option<Box<Value>> = match book.resolve_ref(expr.clone()) {
                Ok(arr2) => Some(Box::new(Value::from(arr2))), 
                _ => None
            }; 
            Value::Range { sheet: sheet.clone(), reference: Reference::from(reference.clone()), value: range_value }
		}, 
        Expr::Func {name, args} => {
            match name.as_str() {
                "OFFSET" => {
                    let offset_value: Value = offset(args, book, debug)?;  
                    match offset_value {
                        Value::Range {sheet: _, reference: _, value } => {
                            Value::from(value.unwrap().as_array2())
                        }, 
                        Value::Error(_) => {
                            return Ok(offset_value); 
                        }, 
                        _ => unreachable!()
                    }
                }, 
                "INDEX" => {
                    index(args, book, debug)?
                }, 
                c => {
                    let arg_values: Vec<Value> = args.into_iter().map(|x| ensure_non_range(evaluate_expr_with_context(x, book, debug).unwrap())).collect::<Vec<Value>>(); 
                    get_function_value(c, arg_values)?
                }
            }
        },
		Expr::Literal(lit) => {
			match lit {
				Literal::Number(f) => Value::from(f), 
				Literal::Boolean(b) => Value::from(b), 
				Literal::Text(s) => Value::from(s)
			}
		},
		Expr::Prefix(p, box_expr) => { 
            let a: Value = ensure_non_range(evaluate_expr_with_context(*box_expr, book, debug)?);
			match p {
				Prefix::Plus => Value::from(a.as_num().abs()),
				Prefix::Minus => a * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
            let a = ensure_non_range(evaluate_expr_with_context(*a, book, debug)?); 
            let b = ensure_non_range(evaluate_expr_with_context(*b, book, debug)?); 
            if a.is_err() {
                return Ok(a); 
            } else if b.is_err() {
                return Ok(b); 
            } else {
                match i {
                    Infix::Plus => a.ensure_single() + b.ensure_single(), 
                    Infix::Minus => a.ensure_single() - b.ensure_single(), 
                    Infix::Multiply => a.ensure_single() * b.ensure_single(), 
                    Infix::Divide => a.ensure_single() / b.ensure_single(), 
                    Infix::Exponent => Exponent {a, b}.evaluate(), 
                    Infix::NotEqual => Value::from(a.ensure_single() != b.ensure_single()), 
                    Infix::Equal => Value::from(a.ensure_single() == b.ensure_single()), 
                    Infix::LessThan => Value::from(a.ensure_single() < b.ensure_single()), 
                    Infix::LessThanEqual => Value::from(a.ensure_single() <= b.ensure_single()), 
                    Infix::GreaterThan => Value::from(a.ensure_single() > b.ensure_single()), 
                    Infix::GreaterThanEqual => Value::from(a.ensure_single() >= b.ensure_single()), 
                    Infix::Ampersand => {
                        let value = if a.is_array() {
                            Value::from(a.as_array().into_iter().map(|x| Value::from(format!("{}{}", x.as_text(), b.as_text()))).collect::<Vec<Value>>())
                        } else if b.is_array() {
                            Value::from(b.as_array().into_iter().map(|x| Value::from(format!("{}{}", a.as_text(), x.as_text()))).collect::<Vec<Value>>())
                        } else {
                            Value::from(format!("{}{}", a.as_text(), b.as_text()))
                        }; 
                        value
                    },
                }
            } 
        }, 
		Expr::Array(x) => Value::Array(x.into_iter().map(|e| ensure_non_range(evaluate_expr_with_context(e, book, debug).unwrap())).collect::<Vec<Value>>()), 
        _ => panic!("Expression {:?} does not convert to a value.", expr)  
	}; 
    if debug {
        match expr.clone() {
            Expr::Literal(_) | Expr::Reference { sheet: _, reference: _} => {}, 
            _ => {
                println!("{} => {:?}", expr, value.clone()); 
            }
        }
    } 
    Ok(value)
}

#[cfg(test)]
mod tests {
	use crate::evaluate::evaluate_str;
    use crate::evaluate::value::Value; 
    use crate::errors::Error; 

    #[test]
    fn test_op_codes() -> Result<(), Error> {
        assert_eq!(evaluate_str(" 1 + 1 ")?, Value::from(2.0));
        assert_eq!(evaluate_str(" 1 - 1 ")?, Value::from(0.0)); 
        assert_eq!(evaluate_str(" 2 * 2 ")?, Value::from(4.0)); 
        assert_eq!(evaluate_str(" (2 + 1) * 2 ")?, Value::from(6.0)); 
        assert_eq!(evaluate_str(" 8 / 4 ")?, Value::from(2.0)); 
        assert_eq!(evaluate_str(" 8^2 ")?, Value::from(64.0)); 
        Ok(())
    }

    #[test]
    fn test_conditionals() -> Result<(), Error> {
        assert_eq!(evaluate_str(" 1=1 ")?, Value::from(true)); 
        Ok(())
    }

    #[test]
    fn test_infix_precedence() -> Result<(), Error> {
        assert_eq!(evaluate_str(" -(1+1)-2 ")?, Value::from(-4.0)); 
        Ok(())
    }

    #[test]
    fn test_formula() -> Result<(), Error> {
        assert_eq!(evaluate_str(" SUM(1, 1) ")?, Value::from(2.0)); 
        assert_eq!(evaluate_str(" SUM(SUM(1, 2), 1) ")?, Value::from(4.0)); 
        Ok(())
    }
}

