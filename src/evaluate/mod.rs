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
			match i {
				Infix::Plus => evaluate_expr(*a)? + evaluate_expr(*b)?, 
				Infix::Minus => evaluate_expr(*a)? - evaluate_expr(*b)?, 
				Infix::Multiply => evaluate_expr(*a)? * evaluate_expr(*b)?, 
				Infix::Divide => evaluate_expr(*a)? / evaluate_expr(*b)?, 
				Infix::Exponent => Exponent {a: evaluate_expr(*a)?, b: evaluate_expr(*b)?}.evaluate(), 
				Infix::NotEqual => Value::from(evaluate_expr(*a)? != evaluate_expr(*b)?), 
				Infix::Equal => Value::from(evaluate_expr(*a)? == evaluate_expr(*b)?), 
				Infix::LessThan => Value::from(evaluate_expr(*a)? < evaluate_expr(*b)?), 
				Infix::LessThanEqual => Value::from(evaluate_expr(*a)? <= evaluate_expr(*b)?), 
				Infix::GreaterThan => Value::from(evaluate_expr(*a)? > evaluate_expr(*b)?), 
				Infix::GreaterThanEqual => Value::from(evaluate_expr(*a)? >= evaluate_expr(*b)?), 
                Infix::Ampersand => {
                    let a = evaluate_expr(*a)?; 
                    let b = evaluate_expr(*b)?; 
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
		}, 
		Expr::Array(x) => Value::Array(x.into_iter().map(|x| evaluate_expr(x).unwrap()).collect::<Vec<Value>>()), 
        Expr::Error(err) => Value::Error(err), 
		_ => panic!("Expression {:?} does not convert to a value.", expr)  
	}; 
    Ok(value)
}

pub fn offset_expr(args: Vec<Expr>, book: &Book) -> Result<Expr, Error> {
    if let Expr::Reference { sheet, reference } = args.get(0).unwrap() { 
        let rows = evaluate_expr_with_context(args.get(1).unwrap().clone(), book)?;
        let cols = evaluate_expr_with_context(args.get(2).unwrap().clone(), book)?; 
        let height = args.get(3); 
        let height_opt: Option<usize> = height.map(|h| {
            evaluate_expr_with_context(h.clone(), book).unwrap().as_num() as usize
        }); 
        let width = args.get(4); 
        let width_opt: Option<usize> = width.map(|w| {
            evaluate_expr_with_context(w.clone(), book).unwrap().as_num() as usize
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

pub fn evaluate_expr_with_context(expr: Expr, book: &Book) -> Result<Value, Error> {
    let value = match expr {
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
                    let offset_value: Value = offset(args, book)?;  
                    match offset_value {
                        Value::Range {sheet: _, reference: _, value } => value.unwrap().as_array2()[[0,0]].clone(), 
                        _ => unreachable!()
                    }
                }, 
                "INDEX" => {
                    index(args, book)?
                }, 
                c => {
                    let arg_values: Vec<Value> = args.into_iter().map(|x| ensure_non_range(evaluate_expr_with_context(x, book).unwrap())).collect::<Vec<Value>>(); 
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
			match p {
				Prefix::Plus => Value::from(ensure_non_range(evaluate_expr_with_context(*box_expr, book)?).as_num().abs()),
				Prefix::Minus => ensure_non_range(evaluate_expr_with_context(*box_expr, book)?) * Value::from(-1.0)
			}
		}, 
		Expr::Infix(i, a, b) => {
			match i {
				Infix::Plus => ensure_non_range(evaluate_expr_with_context(*a, book)?) + ensure_non_range(evaluate_expr_with_context(*b, book)?), 
				Infix::Minus => ensure_non_range(evaluate_expr_with_context(*a, book)?) - ensure_non_range(evaluate_expr_with_context(*b, book)?), 
				Infix::Multiply => ensure_non_range(evaluate_expr_with_context(*a, book)?) * ensure_non_range(evaluate_expr_with_context(*b, book)?), 
				Infix::Divide => ensure_non_range(evaluate_expr_with_context(*a, book)?) / ensure_non_range(evaluate_expr_with_context(*b, book)?), 
				Infix::Exponent => Exponent {a: ensure_non_range(evaluate_expr_with_context(*a, book)?), b: ensure_non_range(evaluate_expr_with_context(*b, book)?)}.evaluate(), 
				Infix::NotEqual => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) != ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
				Infix::Equal => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) == ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
				Infix::LessThan => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) < ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
				Infix::LessThanEqual => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) <= ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
				Infix::GreaterThan => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) > ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
				Infix::GreaterThanEqual => Value::from(ensure_non_range(evaluate_expr_with_context(*a, book)?) >= ensure_non_range(evaluate_expr_with_context(*b, book)?)), 
                Infix::Ampersand => {
                    let a = evaluate_expr_with_context(*a, book)?; 
                    let b = evaluate_expr_with_context(*b, book)?; 
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
		}, 
		Expr::Array(x) => Value::Array(x.into_iter().map(|e| ensure_non_range(evaluate_expr_with_context(e, book).unwrap())).collect::<Vec<Value>>()), 
        _ => panic!("Expression {:?} does not convert to a value.", expr)  
	}; 
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
    fn test_formula() -> Result<(), Error> {
        assert_eq!(evaluate_str(" SUM(1, 1) ")?, Value::from(2.0)); 
        assert_eq!(evaluate_str(" SUM(SUM(1, 2), 1) ")?, Value::from(4.0)); 
        Ok(())
    }
}

