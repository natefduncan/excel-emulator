use nom::branch::*;
use nom::bytes::complete::{take, take_while};
use nom::combinator::{map, verify, opt};
use nom::multi::many0;
use nom::sequence::{preceded, delimited, pair, tuple};
use nom::*;
use nom::Err; 
use nom::error::{Error as NomError, ErrorKind}; 

pub mod ast; 

use crate::lexer::{
    Lexer,
    token::{Token, Tokens}
}; 
use crate::parser::ast::*; 


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

fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
	if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Integer(_) => {
                let (_, t2) = take_while(|c| matches!(c, &Token::Integer(_)) || matches!(c, &Token::Period))(input)?; 
				let mut res = String::new(); 
                for t in t2.tok.iter() {
					res = format!("{}{}", res, t);
                }
                Ok((i1, Literal::Number(res.parse::<f64>().unwrap())))
            },
            Token::Text(s) => Ok((i1, Literal::Text(s))),
            Token::Boolean(b) => Ok((i1, Literal::Boolean(b))),
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag))),
        }
    }
}

fn parse_literal_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::Literal)(input)
}

fn parse_error(input: Tokens) -> IResult<Tokens, Error> {
    let (i1, t1) = take(1usize)(input)?;
	if t1.tok.is_empty() {
        Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Null => Ok((i1, Error::Null)), 
            Token::Div => Ok((i1, Error::Div)), 
            Token::Value => Ok((i1, Error::Value)), 
            Token::Ref => Ok((i1, Error::Ref)), 
            Token::Name => Ok((i1, Error::Name)), 
            Token::Num => Ok((i1, Error::Num)), 
            Token::NA => Ok((i1, Error::NA)), 
            Token::GettingData => Ok((i1, Error::GettingData)), 
            _ => Err(Err::Error(NomError::new(input, ErrorKind::Tag)))
        }
    }
}

fn parse_error_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_error, Expr::Error)(input)
}

fn parse_comma_exprs(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        preceded(alt((comma_tag, semicolon_tag)), parse), 
        |expr| {
            expr
        }
    )(input)
}

fn parse_exprs(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    map(
        pair(parse, many0(parse_comma_exprs)),
        |(first, second)| [&vec![first][..], &second[..]].concat(),
    )(input)
}

fn empty_boxed_vec(input: Tokens) -> IResult<Tokens, Vec<Expr>> {
    Ok((input, vec![]))
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

fn parse_reference(input: Tokens) -> IResult<Tokens, Expr> {
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

fn parse_block(input: Tokens) -> IResult<Tokens, Expr> {
    delimited(lparen_tag, parse, rparen_tag)(input)
}

fn parse_prefix(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        pair(
            alt((plus_tag, minus_tag)), 
            alt((parse_block, parse_expr))
        ),
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

fn parse_infix(input: Tokens) -> IResult<Tokens, Expr> {
    map(
        tuple((
            alt((parse_expr, parse_block)), 
            parse_infix_tags,
            alt((parse_expr, parse_block)), 
        )),
        |(a, infix, b)| {
            Expr::Infix(infix, Box::new(a), Box::new(b))
        }
    )(input)
}
    
fn parse_expr(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_error_expr, 
        parse_func_expr, 
        parse_array_expr, 
        parse_reference, 
        parse_literal_expr, 
    ))(input)
}

pub fn parse(input: Tokens) -> IResult<Tokens, Expr> {
    alt((
        parse_infix, 
        parse_prefix,
        parse_block, 
        parse_expr
    ))(input)
}

pub fn parse_str(s: &str) -> Expr {
    let (_, t) = Lexer::lex_tokens(s.as_bytes()).unwrap(); 
    let tokens = Tokens::new(&t); 
    let (_, expr) = parse(tokens).unwrap(); 
    expr
}

#[cfg(test)]
mod tests {
    use crate::parser::parse; 
    use crate::parser::ast::{Expr, Error, Prefix, Infix}; 
    use crate::lexer::Lexer; 
    use crate::lexer::token::Tokens; 

    fn parse_str(s: &str) -> Expr {
        let (remain, t) = Lexer::lex_tokens(s.as_bytes()).unwrap(); 
        println!("remain: {:?}", remain); 
        println!("tokens: {:?}", t); 
        let tokens = Tokens::new(&t); 
        let (_, expr) = parse(tokens).unwrap(); 
        expr
    }

    #[test]
    fn test_literal() {
        assert_eq!(parse_str("123"), Expr::from(123.0)); 
        assert_eq!(parse_str("123.12"), Expr::from(123.12)); 
        assert_eq!(parse_str("\"Test\""), Expr::from("Test")); 
        assert_eq!(parse_str("TRUE"), Expr::from(true)); 
    }

    #[test]
    fn test_errors() {
        assert_eq!(parse_str("#NULL!"), Expr::Error(Error::Null)); 
        assert_eq!(parse_str("#DIV/0!"), Expr::Error(Error::Div)); 
        assert_eq!(parse_str("#VALUE!"), Expr::Error(Error::Value)); 
        assert_eq!(parse_str("#REF!"), Expr::Error(Error::Ref)); 
        assert_eq!(parse_str("#NAME!"), Expr::Error(Error::Name)); 
        assert_eq!(parse_str("#NUM!"), Expr::Error(Error::Num)); 
        assert_eq!(parse_str("#N/A!"), Expr::Error(Error::NA)); 
        assert_eq!(parse_str("#GETTING_DATA"), Expr::Error(Error::GettingData)); 
    }

    #[test]
    fn test_function() {
        assert_eq!(parse_str("test(\"a\", \"b\")"), Expr::Func {name: String::from("test"), args: vec![Expr::from("a"), Expr::from("b")]}); 
    }

    #[test]
    fn test_reference() {
        assert_eq!(parse_str("test!A1"), Expr::Reference { sheet: Some("test".to_string()), reference: "A1".to_string()}); 
        assert_eq!(parse_str("test!A1:B2"), Expr::Reference { sheet: Some("test".to_string()), reference: "A1:B2".to_string()}); 
    }

    #[test]
    fn test_array() {
        assert_eq!(parse_str("{1, 2, 3, 4}"), Expr::Array(vec![Expr::from(1.0), Expr::from(2.0), Expr::from(3.0), Expr::from(4.0)])); 
        assert_eq!(parse_str("{(1+2), 2, 3, 4}"), Expr::Array(vec![
                Expr::Infix(
                    Infix::Plus, 
                    Box::new(Expr::from(1.0)), 
                    Box::new(Expr::from(2.0))
                ), 
                Expr::from(2.0), 
                Expr::from(3.0), 
                Expr::from(4.0)
        ])); 
    }

    #[test]
    fn test_prefix() {
        assert_eq!(parse_str("+1"), Expr::Prefix(Prefix::Plus, Box::new(Expr::from(1.0)))); 
    }

    #[test]
    fn test_infix() {
        assert_eq!(parse_str("1+1"), Expr::Infix(Infix::Plus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("(1+1)"), Expr::Infix(Infix::Plus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 - 1)"), Expr::Infix(Infix::Minus, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 / 1)"), Expr::Infix(Infix::Divide, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 * 1)"), Expr::Infix(Infix::Multiply, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 ^ 1)"), Expr::Infix(Infix::Exponent, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 = 1)"), Expr::Infix(Infix::Equal, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 < 1)"), Expr::Infix(Infix::LessThan, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <= 1)"), Expr::Infix(Infix::LessThanEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 > 1)"), Expr::Infix(Infix::GreaterThan, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 >= 1)"), Expr::Infix(Infix::GreaterThanEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <> 1)"), Expr::Infix(Infix::NotEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("1 <> 1)"), Expr::Infix(Infix::NotEqual, Box::new(Expr::from(1.0)), Box::new(Expr::from(1.0)))); 
        assert_eq!(parse_str("(1+2)*(3+5)"), Expr::Infix(
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
        assert_eq!(parse_str("(1+(2*1))"), Expr::Infix(
                Infix::Plus, 
                Box::new(Expr::from(1.0)), 
                Box::new(
                    Expr::Infix(
                        Infix::Multiply, 
                        Box::new(Expr::from(2.0)), 
                        Box::new(Expr::from(1.0))
                    )
                ))); 
    }

    #[test]
    fn test_reference_formula() {
        assert_eq!(parse_str("SUM(Sheet1!A1:A10)"), Expr::Func { name: "SUM".to_string(), args: vec![Expr::Reference { sheet: Some("Sheet1".to_string()), reference: "A1:A10".to_string() }] }); 
    }
}
