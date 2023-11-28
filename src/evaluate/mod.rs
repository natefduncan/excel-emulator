use crate::{
    parser::{
        ast::{Literal, Prefix, Infix, Expr, Error as AstError}, 
        parse_str
    }, 
    function::*, 
    evaluate::value::Value, 
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
            let args: Vec<Value> = args.into_iter().map(|x| evaluate_expr(x).unwrap()).collect::<Vec<Value>>(); 
            match name.as_str() {
                "SUM" => Sum::from(args).evaluate(), 
                "SUMIF" => Sumifs::from(args).evaluate(), 
                "AVERAGE" => Average::from(args).evaluate(), 
                "AVERAGEIF" => Averageif::from(args).evaluate(), 
                "COUNT" => Count::from(args).evaluate(),	
                "EXPONENT" => Exponent::from(args).evaluate(),	
                "CONCAT" => Concat::from(args).evaluate(),	
                "AND" => Andfunc::from(args).evaluate(),	
                "OR" => Orfunc::from(args).evaluate(),	
                "MAX" => Max::from(args).evaluate(),	
                "MIN" => Min::from(args).evaluate(),	
                "MATCH" => Matchfn::from(args).evaluate(),	
                "DATE" => Date::from(args).evaluate(),	
                "FLOOR" => Floor::from(args).evaluate(),	
                "IFERROR" => {
                    let a = args.get(0).unwrap().clone(); 
                    let b = args.get(1).unwrap().clone(); 
                    return Ok(Iferror { a, b }.evaluate()); 
                },	
                "EOMONTH" => Eomonth::from(args).evaluate(),	
                "SUMIFS" => Sumifs::from(args).evaluate(),	
                "COUNTIFS" => Countifs::from(args).evaluate(),	
                "AVERAGEIFS" => Averageifs::from(args).evaluate(),	
                "XIRR" => Xirrfunc::from(args).evaluate(),	
                "IF" => Iffunc::from(args).evaluate(),	
                "XNPV" => Xnpv::from(args).evaluate(),	
                "YEARFRAC" => Yearfrac::from(args).evaluate(),	
                "DATEDIF" => Datedif::from(args).evaluate(),	
                "PMT" => Pmt::from(args).evaluate(),	
                "COUNTA" => Counta::from(args).evaluate(),	
                "ROUNDDOWN" => Rounddown::from(args).evaluate(),	
                "ROUNDUP" => Roundup::from(args).evaluate(),	
                "SEARCH" => Search::from(args).evaluate(),	
                "COUNTIF" => Countif::from(args).evaluate(),	
                "MONTH" => Month::from(args).evaluate(),	
                "YEAR" => Year::from(args).evaluate(),	
                "SUMPRODUCT" => Sumproduct::from(args).evaluate(),	
                _ => Value::Error(AstError::Name)
            }
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

