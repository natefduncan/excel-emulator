use nom::branch::*;
use nom::bytes::complete::{tag, take, take_while, take_while1};
use nom::character::complete::{alpha1, alphanumeric1, digit1, multispace0};
use nom::combinator::{map, map_res, recognize, opt};
use nom::multi::many0;
use nom::sequence::{terminated, delimited, separated_pair, pair};
use nom::*;

use std::str;
use std::str::FromStr;
use std::str::Utf8Error;

pub mod token; 
use crate::lexer::token::*; 
use crate::errors::Error; 

macro_rules! syntax {
    ($func_name: ident, $tag_string: literal, $output_token: expr) => {
        fn $func_name(s: &[u8]) -> IResult<&[u8], Token> {
            map(tag($tag_string), |_| $output_token)(s)
        }
    };
}

// Syntax 
syntax! {null_err, "#NULL!", Token::Null}
syntax! {div_err, "#DIV/0!", Token::Div}
syntax! {value_err, "#VALUE!", Token::Value}
syntax! {ref_err, "#REF!", Token::Ref}
syntax! {name_err, "#NAME!", Token::Name}
syntax! {num_err, "#NUM!", Token::Num}
syntax! {na_err, "#N/A", Token::NA}
syntax! {getting_data_err, "#GETTING_DATA", Token::GettingData}
syntax! {plus, "+", Token::Plus}
syntax! {minus, "-", Token::Minus}
syntax! {divide, "/", Token::Divide}
syntax! {multiply, "*", Token::Multiply}
syntax! {exponent, "^", Token::Exponent}
syntax! {ampersand, "&", Token::Ampersand}
syntax! {equal, "=", Token::Equal}
syntax! {comma, ",", Token::Comma}
syntax! {period, ".", Token::Period}
syntax! {colon, ":", Token::Colon}
syntax! {semicolon, ";", Token::SemiColon}
syntax! {langle, "<", Token::LAngle}
syntax! {rangle, ">", Token::RAngle}
syntax! {lparen, "(", Token::LParen}
syntax! {rparen, ")", Token::RParen}
syntax! {lbrace, "{", Token::LBrace}
syntax! {rbrace, "}", Token::RBrace}
syntax! {lbracket, "[", Token::LBracket}
syntax! {rbracket, "]", Token::RBracket}
syntax! {true_bool, "TRUE", Token::Boolean(true)}
syntax! {false_bool , "FALSE", Token::Boolean(false)}

pub fn lex_syntax(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        alt((
            null_err, 
            div_err, 
            value_err, 
            ref_err, 
            name_err, 
            num_err, 
            na_err, 
            getting_data_err
        )), 
        alt((
            plus,
            minus, 
            divide, 
            multiply, 
            exponent 
        )), 
        alt((
            ampersand, 
            equal, 
            comma, 
            colon, 
            period, 
            semicolon, 
            langle, 
            rangle, 
            lparen, 
            rparen, 
            lbrace, 
            rbrace, 
            lbracket, 
            rbracket
        )),
        alt((
            true_bool, 
            false_bool
        ))
    ))(input)
}

// String
fn pis(input: &[u8]) -> IResult<&[u8], Vec<u8>> {
    use std::result::Result::*;

    let (i1, c1) = take(1usize)(input)?;
    match c1.as_bytes() {
        b"\"" => Ok((input, vec![])),
        b"\\" => {
            let (i2, c2) = take(1usize)(i1)?;
            pis(i2).map(|(slice, done)| (slice, concat_slice_vec(c2, done)))
        }
        c => pis(i1).map(|(slice, done)| (slice, concat_slice_vec(c, done))),
    }
}

fn concat_slice_vec(c: &[u8], done: Vec<u8>) -> Vec<u8> {
	let mut new_vec = c.to_vec();
    new_vec.extend(&done);
    new_vec
}

fn convert_vec_utf8(v: Vec<u8>) -> Result<String, Utf8Error> {
	let slice = v.as_slice();
	str::from_utf8(slice).map(|s| s.to_owned())
}

fn complete_byte_slice_str_from_utf8(c: &[u8]) -> Result<&str, Utf8Error> {
    str::from_utf8(c)
}

fn string(input: &[u8]) -> IResult<&[u8], String> {
	delimited(tag("\""), map_res(pis, convert_vec_utf8), tag("\""))(input)
}

fn lex_string(input: &[u8]) -> IResult<&[u8], Token> {
    map(string, Token::Text)(input)
}

// References 
fn lex_vrange(input: &[u8]) -> IResult<&[u8], Token> {
    let vrange_token = recognize(separated_pair(
        pair(opt(tag("$")), alpha1), 
        tag(":"), 
        pair(opt(tag("$")), alpha1), 
    ));
    map_res(
        vrange_token,
        |s| {
            let c = complete_byte_slice_str_from_utf8(s); 
            c.map(|syntax| Token::VRange(syntax.to_string()))
        }
    )(input)
}

fn lex_hrange(input: &[u8]) -> IResult<&[u8], Token> {
    let vrange_token = recognize(separated_pair(
        pair(opt(tag("$")), digit1), 
        tag(":"), 
        pair(opt(tag("$")), digit1), 
    ));
    map_res(
        vrange_token,
        |s| {
            let c = complete_byte_slice_str_from_utf8(s); 
            c.map(|syntax| Token::HRange(syntax.to_string()))
        }
    )(input)
}

fn in_quote_sheet_name(chr: u8) -> bool {
    let is_special = b"`~@#$%^&-_=+{}|;,<.>() ".contains(&chr); 
    is_digit_or_alpha(chr) || is_special
}

fn in_sheet_name(chr: u8) -> bool {
    let is_special = b"`~@#$%^_{}|;.".contains(&chr); 
    is_digit_or_alpha(chr) || is_special
}

fn is_digit(chr: u8) -> bool {
    (0x30..=0x39).contains(&chr)
}

fn is_alpha(chr: u8) -> bool {
    (0x41..=0x5A).contains(&chr) || (0x61..=0x7A).contains(&chr)
}

fn is_digit_or_alpha(chr: u8) -> bool {
    is_digit(chr) || is_alpha(chr)
}

fn lex_sheet_name(input: &[u8]) -> IResult<&[u8], &[u8]> {
    alt((
        take_while1(in_sheet_name),
        recognize(delimited(tag("'"), take_while(in_quote_sheet_name), tag("'")))
    ))(input)
}

fn lex_sheet(input: &[u8]) -> IResult<&[u8], Token> {
    map_res(
        alt((
            terminated(lex_sheet_name, tag("\\!")), // Workaround shell expansion of !. 
            terminated(lex_sheet_name, tag("!")), 
        )), 
        |s| {
            let c = complete_byte_slice_str_from_utf8(s);
            c.map(|syntax| Token::Sheet(syntax.replace('\'', "")))
        }
    )(input)
}

fn lex_multisheet(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        terminated(recognize(separated_pair(lex_sheet_name, tag(":"), lex_sheet_name)), tag("!")), 
        |a| {
            let x = complete_byte_slice_str_from_utf8(a).unwrap();
            Token::MultiSheet(x.to_string())
        }
    )(input)
}

fn lex_cell(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        recognize(pair(pair(opt(tag("$")), alpha1), pair(opt(tag("$")), digit1))), 
        |c| {
            let s = complete_byte_slice_str_from_utf8(c).unwrap(); 
            Token::Cell(s.to_string())
        }
    )(input)
}

fn lex_range(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        separated_pair(lex_cell, tag(":"), lex_cell), 
        |(a, b)| {
            Token::Range(format!("{}:{}", a, b))
        }
    )(input)
}

fn lex_references(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        lex_multisheet,
        lex_sheet, 
        lex_hrange, 
        lex_vrange, 
        lex_range, 
        lex_cell
    ))(input)
}

// Integer
fn complete_str_from_str<F: FromStr>(c: &str) -> Result<F, F::Err> {
    FromStr::from_str(c)
}

fn lex_int(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        map_res(
            map_res(digit1, complete_byte_slice_str_from_utf8),
            complete_str_from_str,
        ),
        Token::Integer,
    )(input)
}

fn lex_float(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        map_res(
            recognize(separated_pair(digit1, period, digit1)),
            complete_byte_slice_str_from_utf8
        ), 
        |c: &str| {
            Token::Float(c.parse::<f64>().unwrap())
        }
    )(input)
}

// Ident
fn lex_ident(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        map_res(
            map_res(alphanumeric1, complete_byte_slice_str_from_utf8),
            complete_str_from_str,
        ),
        Token::Ident,
    )(input)
}

// Tokens
fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        lex_syntax,
        lex_string,
        lex_references,
        lex_float, 
        lex_int, 
        lex_ident, 
    ))(input)
}

fn lex_tokens(input: &[u8]) -> IResult<&[u8], Vec<Token>> {
    many0(delimited(multispace0, lex_token, multispace0))(input)
}

pub struct Lexer; 
impl Lexer {
    pub fn lex_tokens(bytes: &[u8]) -> Result<Vec<Token>, Error> {
		match lex_tokens(bytes)
			.map(|(slice, result)| (slice, [&result[..]].concat())) {
                Ok((_, mut tokens)) => {
                    tokens.push(Token::EOF); 
                    Ok(tokens)
                }, 
                _ => Err(Error::UnableToLex(String::from_utf8(bytes.to_vec()).unwrap()))
            }
	}
}

#[cfg(test)]
mod tests {
	use super::*; 
    use crate::errors::Error; 

	fn lex(b: &[u8]) -> Result<Vec<Token>, Error> {
        Lexer::lex_tokens(b)
    }

	#[test]
	fn test_symbols() -> Result<(), Error> {
        assert_eq!(lex(b"=+(){},;")?, vec![
			Token::Equal, 
            Token::Plus,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::Comma,
            Token::SemiColon,
            Token::EOF, 
        ]);
        Ok(())
	}

    #[test]
    fn test_strings() -> Result<(), Error> {
        assert_eq!(lex(b"\"this is a test\"")?, vec![
            Token::Text(String::from("this is a test")), Token::EOF
        ]);
        assert_eq!(lex(b"\"this\", \"is\" \"a\" \"test\"")?, vec![
            Token::Text(String::from("this")), 
            Token::Comma, 
            Token::Text(String::from("is")), 
            Token::Text(String::from("a")), 
            Token::Text(String::from("test")), 
            Token::EOF, 
        ]);
        Ok(())
    }

    #[test]
    fn test_ints() -> Result<(), Error> {
        assert_eq!(lex(b"123")?, vec![
            Token::Integer(123), Token::EOF
        ]); 
        assert_eq!(lex(b"0.05")?, vec![
            Token::Float(0.05), Token::EOF
        ]); 
        assert_eq!(lex(b"12.30")?, vec![
            Token::Float(12.30), Token::EOF
        ]); 
        Ok(())
    }

    #[test]
    fn test_errors() -> Result<(), Error> {
        assert_eq!(lex(b"#NUM!")?, vec![Token::Num, Token::EOF]); 
        assert_eq!(lex(b"#DIV/0!")?, vec![Token::Div, Token::EOF]); 
        assert_eq!(lex(b"#VALUE!")?, vec![Token::Value, Token::EOF]); 
        assert_eq!(lex(b"#REF!")?, vec![Token::Ref, Token::EOF]); 
        assert_eq!(lex(b"#NAME!")?, vec![Token::Name, Token::EOF]); 
        assert_eq!(lex(b"#N/A")?, vec![Token::NA, Token::EOF]); 
        assert_eq!(lex(b"#GETTING_DATA")?, vec![Token::GettingData, Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_bool() -> Result<(), Error> {
        assert_eq!(lex(b"TRUE")?, vec![Token::Boolean(true), Token::EOF]); 
        assert_eq!(lex(b"FALSE")?, vec![Token::Boolean(false), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_multisheet() -> Result<(), Error> {
        assert_eq!(lex(b"test:test!")?, vec![Token::MultiSheet(String::from("test:test")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_sheet() -> Result<(), Error> {
        assert_eq!(lex(b"'Test'!")?, vec![Token::Sheet(String::from("Test")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_vrange() -> Result<(), Error> {
        assert_eq!(lex(b"A:A")?, vec![Token::VRange(String::from("A:A")), Token::EOF]); 
        assert_eq!(lex(b"$A:$A")?, vec![Token::VRange(String::from("$A:$A")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_hrange() -> Result<(), Error> {
        assert_eq!(lex(b"1:1")?, vec![Token::HRange(String::from("1:1")), Token::EOF]); 
        assert_eq!(lex(b"$1:$1")?, vec![Token::HRange(String::from("$1:$1")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_range() -> Result<(), Error> {
        assert_eq!(lex(b"A1:A1")?, vec![Token::Range(String::from("A1:A1")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_cell() -> Result<(), Error> {
        assert_eq!(lex(b"A1")?, vec![Token::Cell(String::from("A1")), Token::EOF]); 
        Ok(())
    }

    #[test]
    fn test_ident() -> Result<(), Error> {
        assert_eq!(lex(b"test")?, vec![Token::Ident("test".to_string()), Token::EOF]); 
        Ok(())
    }
}
