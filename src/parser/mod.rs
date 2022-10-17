use nom::branch::*;
use nom::bytes::complete::take;
use nom::combinator::{map, verify, opt};
use nom::multi::many0;
use nom::sequence::{preceded, delimited, pair, terminated};
use nom::*;
use nom::Err; 
use nom::error::{Error as NomError, ErrorKind}; 

pub mod ast; 

use crate::{
    lexer::{
        Lexer,
        token::{Token, Tokens}, 
    }, 
    parser::ast::{Expr, Error as ExcelError, Literal, Prefix, Infix, Precedence}, 
    errors::Error
}; 

macro_rules! tag_token (
	($func_name:ident, $tag: expr) => (
		fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
			verify(take(1usize), |t: &Tokens| t.tok[0] == $tag)(tokens)
		}
	)
);

tag_token!(comma_tag, Token::Comma); 
tag_token!(plus_tag, Token::Plus); 
tag_token!(minus_tag, Token::Minus); 
tag_token!(divide_tag, Token::Divide); 
tag_token!(multiply_tag, Token::Multiply); 
tag_token!(exponent_tag, Token::Exponent); 
tag_token!(ampersand_tag, Token::Ampersand); 
tag_token!(equal_tag, Token::Equal); 
tag_token!(semicolon_tag, Token::SemiColon); 
tag_token!(langle_tag, Token::LAngle); 
tag_token!(rangle_tag, Token::RAngle); 
tag_token!(lparen_tag, Token::LParen); 
tag_token!(rparen_tag, Token::RParen); 
tag_token!(lbrace_tag, Token::LBrace); 
tag_token!(rbrace_tag, Token::RBrace); 
tag_token!(eof_tag, Token::EOF); 

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
	if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Integer(x) => Ok((i1, Literal::Number(x as f64))), 
            Token::Float(x) => Ok((i1, Literal::Number(x))), 
            Token::Text(s) => Ok((i1, Literal::Text(s))),
            Token::Boolean(b) => Ok((i1, Literal::Boolean(b))),
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_literal_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::Literal)(input)
}

fn parse_error(input: Tokens) -> IResult<Tokens, ExcelError> {
    let (i1, t1) = take(1usize)(input)?;
	if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Null => Ok((i1, ExcelError::Null)), 
            Token::Div => Ok((i1, ExcelError::Div)), 
            Token::Value => Ok((i1, ExcelError::Value)), 
            Token::Ref => Ok((i1, ExcelError::Ref)), 
            Token::Name => Ok((i1, ExcelError::Name)), 
            Token::Num => Ok((i1, ExcelError::Num)), 
            Token::NA => Ok((i1, ExcelError::NA)), 
            Token::GettingData => Ok((i1, ExcelError::GettingData)), 
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
        }
    }
}

fn parse_error_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_error, Expr::Error)(input)
}

fn parse_ident(input: Tokens) -> IResult<Tokens, Token> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else if matches!(t1.tok[0], Token::Ident(_)) {
        Ok((i1, t1.tok[0].clone()))
    } else {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    }
}

fn parse_func_expr(input: Tokens) -> IResult<Tokens, Expr> {
   map(
       pair(
           parse_ident, 
           delimited(
               lparen_tag,
               alt((parse_exprs, empty_boxed_vec)),
               rparen_tag,
           )
        ),
        |(ident, exprs)| {
            Expr::Func { name: format!("{}", ident), args: exprs }
        }
   )(input)
}

fn parse_prefix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        pair(alt((plus_tag, minus_tag)), parse_atom_expr), 
        |(pre, expr)| {
            let prefix = match &pre.tok[0] {
                Token::Plus => Prefix::Plus, 
                Token::Minus => Prefix::Minus, 
                _ => unreachable!()
            }; 
            let box_expr = Box::new(expr); 
            Expr::Prefix(prefix, box_expr)
        }
    )(input)
}


fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        preceded(alt((comma_tag, semicolon_tag)), parse_expr), 
        |expr| {
            expr
        }
    )(input)
}

fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    map(
        pair(parse_expr, many0(parse_comma_exprs)),
        |(first, second)| {
            [&vec![first][..], &second[..]].concat()
        }
    )(input)
}

fn empty_boxed_vec(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    Ok((input, vec![]))
}

fn parse_array_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        delimited(
            lbrace_tag, 
            alt((parse_exprs, empty_boxed_vec)),
            rbrace_tag,
        ), 
        |exprs| {
            Expr::Array(exprs)
        }
    )(input)
}

fn parse_sheet_or_multisheet(input: Tokens) -> IResult<Tokens, Token> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match &t1.tok[0] {
            Token::MultiSheet(s) => Ok((i1, Token::MultiSheet(s.to_string()))), 
            Token::Sheet(s) => Ok((i1, Token::Sheet(s.to_string()))), 
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
        }
    }
}

fn parse_cell_or_range(input: Tokens) -> IResult<Tokens, Token> {
    let (i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match &t1.tok[0] {
            Token::Range(s) => Ok((i1, Token::Range(s.to_string()))), 
            Token::Cell(s) => Ok((i1, Token::Cell(s.to_string()))), 
            Token::VRange(s) => Ok((i1, Token::VRange(s.to_string()))), 
            Token::HRange(s) => Ok((i1, Token::HRange(s.to_string()))), 
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
        }
    }
}

fn parse_reference_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        pair(
            opt(parse_sheet_or_multisheet), parse_cell_or_range
        ), 
        |(sheet, range)| {
            let sheet : Option<String> = sheet.map(|x| format!("{}", x));
            Expr::Reference {
                sheet, reference: format!("{}", range)
            }
       }
    )(input)
}


fn parse_paren_expr(input: Tokens) -> IResult<Tokens, Expr> {
    delimited(lparen_tag, parse_expr, rparen_tag)(input)
}


fn infix_precedence(infix: Infix) -> Precedence {
    match infix {
        Infix::Equal
            | Infix::NotEqual 
            | Infix::LessThan
            | Infix::LessThanEqual
            | Infix::GreaterThan
            | Infix::GreaterThanEqual => Precedence::Comparison, 
        Infix::Ampersand => Precedence::Concat, 
        Infix::Plus | Infix::Minus => Precedence::PlusMinus, 
        Infix::Multiply | Infix::Divide => Precedence::MultDiv, 
        Infix::Exponent => Precedence::Exponent, 
    }
}

fn parse_infix_tags(input: Tokens) -> IResult<Tokens, Infix> {
    alt((
        map(plus_tag, |_| Infix::Plus), 
        map(minus_tag, |_| Infix::Minus), 
        map(divide_tag, |_| Infix::Divide), 
        map(multiply_tag, |_| Infix::Multiply),   
        map(equal_tag, |_| Infix::Equal), 
        map(pair(langle_tag, rangle_tag), |(_, _)| Infix::NotEqual), 
        map(pair(langle_tag, equal_tag), |(_, _)| Infix::LessThanEqual), 
        map(pair(rangle_tag, equal_tag), |(_, _)| Infix::GreaterThanEqual), 
        map(rangle_tag, |_| Infix::GreaterThan), 
        map(langle_tag, |_| Infix::LessThan), 
        map(exponent_tag, |_| Infix::Exponent), 
        map(ampersand_tag, |_| Infix::Ampersand), 
    ))(input)
}

fn parse_pratt(input: Tokens, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, left) = parse_atom_expr(input)?;
    go_parse_pratt(i1, left, precedence)
}

fn go_parse_pratt(input: Tokens, lhs: Expr, precedence: Precedence) -> IResult<Tokens, Expr> {
    let (i1, t1) = take(1usize)(input)?; 
    if t1.tok.is_empty() {
        Ok((i1, lhs))
    } else {
        match t1.tok[0] {
            Token::EOF => Ok((input, lhs)), 
            _ => {
                match parse_infix_tags(input) {
                    Ok((_, infix)) => {
                        let p = infix_precedence(infix); 
                        if precedence < p {
                            let (i2, lhs2) = parse_infix(input, lhs)?;
                            go_parse_pratt(i2, lhs2, precedence) 
                        } else {
                            Ok((input, lhs))
                        }
                    }, 
                    _ => Ok((input, lhs))
                }
           }
        }
    }
}

fn parse_infix(input: Tokens, lhs: Expr) -> IResult<Tokens, Expr> {
    let (_i1, t1) = take(1usize)(input)?;
    if t1.tok.is_empty() {
        Err(Err::Error(error_position!(input, ErrorKind::Tag)))
    } else {
        let (i2, infix) = parse_infix_tags(input)?;
        let p = infix_precedence(infix.clone()); 
        let (i3, rhs) = parse_pratt(i2, p)?;
        Ok((i3, Expr::Infix(infix, Box::new(lhs), Box::new(rhs))))
    }
}

fn parse_infix_expr(input: Tokens) -> IResult<Tokens, Expr> {
    parse_pratt(input, Precedence::Lowest)
}

fn parse_atom_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_prefix_expr,
        parse_paren_expr, 
        parse_error_expr, 
        parse_func_expr, 
        parse_array_expr, 
        parse_reference_expr, 
        parse_literal_expr, 
    ))(input)
}

fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_infix_expr, 
        parse_atom_expr,
    ))(input)
}

pub fn parse(input: Tokens) -> IResult<Tokens, Expr> {
    terminated(parse_expr, eof_tag)(input)
}

pub fn parse_str(s: &str) -> Result<Expr, Error> {
    let t = Lexer::lex_tokens(s.as_bytes())?; 
    let tokens = Tokens::new(&t); 
    match parse(tokens) {
        Ok((_, expr)) => Ok(expr),
        _ => Err(Error::UnableToParse(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_str; 
    use crate::parser::ast::{Expr, Error as ExcelError, Prefix, Infix}; 
    use crate::errors::Error; 

    #[test]
    fn test_literal() -> Result<(), Error> {
        assert_eq!(parse_str("123")?, Expr::from(123.0)); 
        assert_eq!(parse_str("1")?, Expr::from(1.0)); 
        assert_eq!(parse_str("123.12")?, Expr::from(123.12)); 
        assert_eq!(parse_str("123.123")?, Expr::from(123.123)); 
        assert_eq!(parse_str("\"Test\"")?, Expr::from("Test")); 
        assert_eq!(parse_str("TRUE")?, Expr::from(true)); 
        Ok(())
    }

    #[test]
    fn test_errors() -> Result<(), Error> {
        assert_eq!(parse_str("#NULL!")?, Expr::Error(ExcelError::Null)); 
        assert_eq!(parse_str("#DIV/0!")?, Expr::Error(ExcelError::Div)); 
        assert_eq!(parse_str("#VALUE!")?, Expr::Error(ExcelError::Value)); 
        assert_eq!(parse_str("#REF!")?, Expr::Error(ExcelError::Ref)); 
        assert_eq!(parse_str("#NAME!")?, Expr::Error(ExcelError::Name)); 
        assert_eq!(parse_str("#NUM!")?, Expr::Error(ExcelError::Num)); 
        assert_eq!(parse_str("#N/A!")?, Expr::Error(ExcelError::NA)); 
        assert_eq!(parse_str("#GETTING_DATA")?, Expr::Error(ExcelError::GettingData)); 
        Ok(())
    }

    #[test]
    fn test_function() -> Result<(), Error> {
        assert_eq!(parse_str("test(\"a\", \"b\")")?, Expr::Func {name: String::from("test"), args: vec![Expr::from("a"), Expr::from("b")]}); 
        Ok(())
    }

    #[test]
    fn test_reference() -> Result<(), Error> {
        assert_eq!(parse_str("test!A1")?, Expr::Reference { sheet: Some("test".to_string()), reference: "A1".to_string()}); 
        assert_eq!(parse_str("test!A1:B2")?, Expr::Reference { sheet: Some("test".to_string()), reference: "A1:B2".to_string()}); 
        Ok(())
    }

    #[test]
    fn test_array() -> Result<(), Error> {
        assert_eq!(parse_str("{1, 2, 3, 4}")?, Expr::Array(vec![Expr::from(1.0), Expr::from(2.0), Expr::from(3.0), Expr::from(4.0)])); 
        assert_eq!(parse_str("{(1+2), 2, 3, 4}")?, Expr::Array(vec![
                Expr::Infix(
                    Infix::Plus, 
                    Box::new(Expr::from(1.0)), 
                    Box::new(Expr::from(2.0))
                ), 
                Expr::from(2.0), 
                Expr::from(3.0), 
                Expr::from(4.0)
        ])); 
        Ok(())
    }

    #[test]
    fn test_prefix() -> Result<(), Error>{
        assert_eq!(parse_str("+1")?, Expr::Prefix(Prefix::Plus, Box::new(Expr::from(1.0)))); 
        Ok(())
    }

    #[test]
    fn test_infix() -> Result<(), Error> {
        assert_eq!(parse_str("1+1")?, Expr::Infix(Infix::Plus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("(1+1)")?, Expr::Infix(Infix::Plus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 - 1")?, Expr::Infix(Infix::Minus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 / 1")?, Expr::Infix(Infix::Divide, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 ^ 1")?, Expr::Infix(Infix::Exponent, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 * 1")?, Expr::Infix(Infix::Multiply, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 = 1")?, Expr::Infix(Infix::Equal, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 < 1")?, Expr::Infix(Infix::LessThan, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <= 1")?, Expr::Infix(Infix::LessThanEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 > 1")?, Expr::Infix(Infix::GreaterThan, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 >= 1")?, Expr::Infix(Infix::GreaterThanEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <> 1")?, Expr::Infix(Infix::NotEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <> 1")?, Expr::Infix(Infix::NotEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("(1+2)*(3+5)")?, Expr::Infix(
                Infix::Multiply, 
                Box::new(Expr::Infix(
                        Infix::Plus,
                        Box::new(Expr::from(1.0)), 
                        Box::new(Expr::from(2.0))
                )), 
                Box::new(Expr::Infix(
                        Infix::Plus, 
                        Box::new(Expr::from(3.0)), 
                        Box::new(Expr::from(5.0))
                ))
        )); 
        assert_eq!(parse_str("(1+(2*1))")?, Expr::Infix(
                Infix::Plus, 
                Box::new(Expr::from(1.0)), 
                Box::new(
                    Expr::Infix(
                        Infix::Multiply, 
                        Box::new(Expr::from(2.0)), 
                        Box::new(Expr::from(1.0))
                    )
                ))); 
        Ok(())
    }

    #[test]
    fn test_reference_formula() -> Result<(), Error> {
        assert_eq!(parse_str("SUM(Sheet1!A1:A10)")?, Expr::Func { name: "SUM".to_string(), args: vec![Expr::Reference { sheet: Some("Sheet1".to_string()), reference: "A1:A10".to_string() }] }); 
        Ok(())
    }

    #[test]
    fn test_floor() -> Result<(), Error> {
        assert_eq!(parse_str("FLOOR(3.7, 1)")?, Expr::Func {
            name: "FLOOR".to_string(),
            args: vec![
                Expr::from(3.7), 
                Expr::from(1.0)
            ]
        }); 
        Ok(())
    }

    #[test]
    fn test_infix_precedence() -> Result<(), Error> {
        assert_eq!(parse_str("-(1+1)-2")?, Expr::Infix(
                Infix::Minus, 
                Box::new(Expr::Prefix(
                        Prefix::Minus, 
                        Box::new(Expr::Infix(
                                Infix::Plus, 
                                Box::new(Expr::from(1.0)), 
                                Box::new(Expr::from(1.0))
                        ))
                )), 
                Box::new(Expr::from(2.0))
        )); 
        Ok(())
    }

    #[test]
    fn test_complex() -> Result<(), Error> {
        assert_eq!(parse_str("1*1*1*1")?, 
            Expr::Infix(
                Infix::Multiply, 
                Box::new(
                    Expr::Infix(
                        Infix::Multiply,
                        Box::new(
                            Expr::Infix(
                                Infix::Multiply, 
                                Box::new(Expr::from(1.0)),
                                Box::new(Expr::from(1.0))
                            )
                        ), 
                        Box::new(Expr::from(1.0))
                    ) 
                ), 
                Box::new(Expr::from(1.0))
            )
        ); 
        Ok(())
    }
}
