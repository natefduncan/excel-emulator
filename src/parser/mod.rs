use nom::branch::*;
use nom::bytes::complete::{tag, take, take_while};
use nom::character::complete::{digit1, multispace0};
use nom::combinator::{map, map_res, verify};
use nom::multi::many0;
use nom::sequence::delimited;
use nom::*;
use nom::Err; 
use nom::error::{Error, ErrorKind}; 

pub mod ast; 

// use crate::parse::expression::{Expr, Error}; 
use crate::lexer::token::{Token, Tokens}; 
use crate::parser::ast::{Literal, Error as ExcelError, Expr, Infix, Prefix}; 

macro_rules! tag_token (
	($func_name:ident, $tag: expr) => (
		fn $func_name(tokens: Tokens) -> IResult<Tokens, Tokens> {
			verify(take(1usize), |t: &Tokens| t.tok[0] == $tag)(tokens)
		}
	)
);

pub fn parse_literal(input: Tokens) -> IResult<Tokens, Literal> {
    let (i1, t1) = take(1usize)(input)?;
	if t1.tok.is_empty() {
        Err(Err::Error(Error::new(input, ErrorKind::Tag)))
    } else {
        match t1.tok[0].clone() {
            Token::Integer(i) => {
                let (i2, t2) = take_while(|c| matches!(c, &Token::Integer(_)) || matches!(c, &Token::Period))(input)?; 
				let mut res = String::new(); 
                for t in t2.tok.into_iter() {
					res = format!("{}{}", res, t);
                }
                Ok((i1, Literal::Number(res.parse::<f64>().unwrap())))
            },
            Token::Text(s) => Ok((i1, Literal::Text(s))),
            Token::Boolean(b) => Ok((i1, Literal::Boolean(b))),
            _ => Err(Err::Error(Error::new(input, ErrorKind::Tag))),
        }
    }
}

tag_token!(plus_tag, Token::Plus); 
tag_token!(minus_tag, Token::Minus); 
tag_token!(divide_tag, Token::Divide); 
tag_token!(multiply_tag, Token::Multiply); 
tag_token!(exponent_tag, Token::Exponent); 
tag_token!(ampersand_tag, Token::Ampersand); 
tag_token!(equal_tag, Token::Equal); 
tag_token!(exclamation_tag, Token::Exclamation); 
tag_token!(comma_tag, Token::Comma); 
tag_token!(period_tag, Token::Period); 
tag_token!(colon_tag, Token::Colon); 
tag_token!(semicolon_tag, Token::SemiColon); 
tag_token!(langle_tag, Token::LAngle); 
tag_token!(rangle_tag, Token::RAngle); 
tag_token!(lparen_tag, Token::LParen); 
tag_token!(rparen_tag, Token::RParen); 
tag_token!(lbrace_tag, Token::LBrace); 
tag_token!(rbrace_tag, Token::RBrace); 
tag_token!(lbracket_tag, Token::LBracket); 
tag_token!(rbracket_tag, Token::RBracket); 

fn parse_literal_expr(input: Tokens) -> IResult<Tokens, Expr> {
    map(parse_literal, Expr::Literal)(input)
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_literal_expr; 
    use crate::parser::ast::{Expr, Literal}; 
    use crate::lexer::Lexer; 
    use crate::lexer::token::{Token, Tokens}; 

    fn parse_expr(s: &str) -> Expr {
        let (_, t) = Lexer::lex_tokens(s.as_bytes()).unwrap(); 
        let tokens = Tokens::new(&t); 
        let (tokens, expr) = parse_literal_expr(tokens).unwrap(); 
        expr
    }

    #[test]
    fn test_literal() {
        assert_eq!(parse_expr("123"), Expr::Literal(Literal::Number(123.0))); 
        assert_eq!(parse_expr("123.12"), Expr::Literal(Literal::Number(123.12))); 
    }
}

// #[cfg(test)]
// mod tests {
    // use crate::excel::*; 
    // fn parse_expr(expr: &str) -> String {
        // println!("{}", expr); 
        // println!("{:?}", ExprParser::new().parse(expr).unwrap()); 
        // format!("{}", ExprParser::new().parse(expr).unwrap())
    // }

    // #[test]
    // fn test_num() {
        // assert_eq!(&parse_expr(" 1 "), "1"); 
        // assert_eq!(&parse_expr(" 150 "), "150"); 
    // }

    // #[test]
    // fn test_operators() {
        // assert_eq!(&parse_expr("1 + 1"), "(1+1)"); 
        // assert_eq!(&parse_expr("1 - 1"), "(1-1)");
        // assert_eq!(&parse_expr("1 / 1"), "(1/1)");
        // assert_eq!(&parse_expr("1 * 1"), "(1*1)");
        // assert_eq!(&parse_expr("1 ^ 1"), "(1^1)");
        // assert_eq!(&parse_expr("1 = 1"), "(1=1)");
        // assert_eq!(&parse_expr("1 < 1"), "(1<1)");
        // assert_eq!(&parse_expr("1 <= 1"), "(1<=1)");
        // assert_eq!(&parse_expr("1 > 1"), "(1>1)");
        // assert_eq!(&parse_expr("1 >= 1"), "(1>=1)");
        // assert_eq!(&parse_expr("1 <> 1"), "(1<>1)");
        // assert_eq!(&parse_expr("1 % 1"), "(1%1)");
        // assert_eq!(&parse_expr("22 * 44 + 66"), "((22*44)+66)");
        // assert_eq!(&parse_expr("(1+2)*(3+5)"), "((1+2)*(3+5))");
    // }

    // #[test] 
    // fn test_errors() {
        // assert_eq!(&parse_expr("#NULL!"), "#NULL!");
        // assert_eq!(&parse_expr("#DIV/0!"), "#DIV/0!");
        // assert_eq!(&parse_expr("#VALUE!"), "#VALUE!");
        // assert_eq!(&parse_expr("#REF!"), "#REF!");
        // assert_eq!(&parse_expr("#NAME?"), "#NAME?");
        // assert_eq!(&parse_expr("#NUM!"), "#NUM!");
        // assert_eq!(&parse_expr("#N/A"), "#N/A");
        // assert_eq!(&parse_expr("#GETTING_DATA"), "#GETTING_DATA");
    // }

    // #[test]
    // fn test_cell() {
        // assert_eq!(&parse_expr(" Sheet!A1 "), "Sheet!A1");
        // assert_eq!(&parse_expr(" 'Sheet 1'!A1 "), "'Sheet 1'!A1");
        // assert_eq!(&parse_expr(" 'Sheet 1':'Sheet 2'!A1 "), "'Sheet 1':'Sheet 2'!A1");
        // assert_eq!(&parse_expr(" Sheet1:Sheet2!A1 "), "Sheet1:Sheet2!A1");
        // assert_eq!(&parse_expr(" Sheet1!A1:A2 "), "Sheet1!A1:A2");
        // assert_eq!(&parse_expr(" Sheet1!$A$1:A2 "), "Sheet1!$A$1:A2");
        // assert_eq!(&parse_expr(" A1 "), "A1");
    // }

    // #[test]
    // fn test_bool() {
        // assert_eq!(&parse_expr(" TRUE "), "TRUE"); 
        // assert_eq!(&parse_expr(" FALSE "), "FALSE"); 
    // }

    // #[test]
    // fn test_text() {
        // assert_eq!(&parse_expr(" \" TEST \" "), "\" TEST \"");
    // }

    // #[test]
    // fn test_function() {
        // assert_eq!(&parse_expr(" test('a', 'b') "), "test(\"a\", \"b\")"); 
    // }

    // #[test]
    // fn test_array() {
        // assert_eq!(&parse_expr(" {1, 2, 3, 4} "), "{1, 2, 3, 4}"); 
        // assert_eq!(&parse_expr(" {1, 2; 3, 4} "), "{1, 2, 3, 4}"); 
    // }

    // #[test]
    // fn test_mix() {
        // assert_eq!(&parse_expr("test({1, 2, 3, 4}, 1, 'a')"), "test({1, 2, 3, 4}, 1, \"a\")");
        // assert_eq!(&parse_expr("(1+2)+(10+test!A1)"), "((1+2)*(10+test!A1))"); 
    // }
// }
