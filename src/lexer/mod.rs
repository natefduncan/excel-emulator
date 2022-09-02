use nom::branch::*;
use nom::bytes::complete::{tag, take};
use nom::character::complete::{alpha1, alphanumeric1, digit1, multispace0};
use nom::combinator::{map, map_res, recognize};
use nom::multi::many0;
use nom::sequence::{delimited, pair};
use nom::*;

use std::str;
use std::str::FromStr;
use std::str::Utf8Error;

pub mod token; 
use crate::lexer::token::*; 

macro_rules! syntax {
    ($func_name: ident, $tag_string: literal, $output_token: expr) => {
        fn $func_name<'a>(s: &'a [u8]) -> IResult<&[u8], Token> {
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
syntax! {exclamation, "!", Token::Exclamation}
syntax! {comma, ",", Token::Comma}
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
                    exclamation, 
                    comma, 
                    colon, 
                    semicolon, 
                    langle, 
                    rangle, 
                    lparen, 
                    rparen, 
                    lbrace, 
                    rbrace, 
                    lbracket, 
                    rbracket
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

// Integer
fn complete_str_from_str<F: FromStr>(c: &str) -> Result<F, F::Err> {
    FromStr::from_str(c)
}

fn lex_integer(input: &[u8]) -> IResult<&[u8], Token> {
    map(
        map_res(
            map_res(digit1, complete_byte_slice_str_from_utf8),
            complete_str_from_str,
        ),
        Token::Integer,
    )(input)
}

// Tokens
fn lex_token(input: &[u8]) -> IResult<&[u8], Token> {
    alt((
        lex_syntax,
        lex_string,
        lex_integer,
    ))(input)
}

fn lex_tokens(input: &[u8]) -> IResult<&[u8], Vec<Token>> {
    many0(delimited(multispace0, lex_token, multispace0))(input)
}


pub struct Lexer; 
impl Lexer {
    pub fn lex_tokens(bytes: &[u8]) -> IResult<&[u8], Vec<Token>> {
		lex_tokens(bytes)
			.map(|(slice, result)| (slice, [&result[..]].concat()))
	}
}

#[cfg(test)]
mod tests {
	use super::*; 

	#[test]
	fn test_lexer1() {
        let input = &b"=+(){},;";
        let (_, result) = Lexer::lex_tokens(input).unwrap();

        let expected_results = vec![
			Token::Equal, 
            Token::Plus,
            Token::LParen,
            Token::RParen,
            Token::LBrace,
            Token::RBrace,
            Token::Comma,
            Token::SemiColon,
        ];

        assert_eq!(result, expected_results);
	}
}
