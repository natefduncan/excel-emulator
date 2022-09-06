use nom::*; 
use std::iter::Enumerate;
use std::ops::{Range, RangeFrom, RangeFull, RangeTo};
use std::fmt; 

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum Token {
    //Literal
    Integer(i64),
    Boolean(bool), 
    Text(String), 
    // Error
    Null, 
    Div, 
    Value, 
    Ref, 
    Name, 
    Num, 
    NA, 
    GettingData, 
    // References
    MultiSheet(String), 
    Sheet(String), 
    Range(String), 
    Cell(String), 
    VRange(String), 
    HRange(String), 
    // Symbols
    Plus,
    Minus,
    Divide,
    Multiply,
    Exponent, 
    Ampersand, 
    Equal,
	Exclamation, 
    Comma,
    Period, 
    Colon,
    SemiColon,
    LAngle,
    RAngle, 
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Ident(String), 
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Token::Integer(i) => write!(f, "{}", i), 
            Token::Boolean(b) => {
                if *b {
                    write!(f, "TRUE")
                } else {
                    write!(f, "FALSE")
                }
            }, 
            Token::Text(s) => write!(f, "{}", s), 
            Token::MultiSheet(s) => write!(f, "{}", s), 
            Token::Sheet(s) => write!(f, "{}", s), 
            Token::Range(s) => write!(f, "{}", s), 
            Token::Cell(s) => write!(f, "{}", s), 
            Token::VRange(s) => write!(f, "{}", s), 
            Token::HRange(s) => write!(f, "{}", s), 
            Token::Ident(s) => write!(f, "{}", s), 
            Token::Null => write!(f, "#NULL!"), 
            Token::Div => write!(f, "#DIV/0!"), 
            Token::Value => write!(f, "#VALUE!"),
            Token::Ref => write!(f, "#REF!"), 
            Token::Name => write!(f, "#NAME!"), 
            Token::Num => write!(f, "#NUM!"), 
            Token::NA => write!(f, "#N/A!"), 
            Token::GettingData => write!(f, "#GETTING_DATA"), 
            Token::Plus => write!(f, "+"), 
            Token::Minus => write!(f, "-"), 
            Token::Divide => write!(f, "/"), 
            Token::Multiply => write!(f, "*"), 
            Token::Exponent => write!(f, "^"), 
            Token::Ampersand => write!(f, "&"), 
            Token::Equal => write!(f, "="), 
            Token::Exclamation => write!(f, "!"), 
            Token::Comma => write!(f, ","), 
            Token::Period => write!(f, "."), 
            Token::Colon => write!(f, ":"), 
            Token::SemiColon => write!(f, ";"), 
            Token::LAngle => write!(f, "<"), 
            Token::RAngle => write!(f, ">"), 
            Token::LParen => write!(f, "("), 
            Token::RParen => write!(f, ")"), 
            Token::LBrace => write!(f, "{{"), 
            Token::RBrace => write!(f, "}}"), 
            Token::LBracket => write!(f, "["), 
            Token::RBracket => write!(f, "]")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Tokens<'a> {
    pub tok: &'a [Token], 
    pub start: usize, 
    pub end: usize, 
}

impl<'a> Tokens<'a> {
    pub fn new(vec: &'a [Token]) -> Self {
        Tokens {
            tok: vec,
            start: 0,
            end: vec.len(),
        }
    }
}

impl<'a> InputLength for Tokens<'a> {
    fn input_len(&self) -> usize {
        self.tok.len()
    }
}

impl<'a> InputTake for Tokens<'a> {
    fn take(&self, count: usize) -> Self {
        Tokens {
            tok: &self.tok[0..count],
            start: 0,
            end: count,
        }
    }

    fn take_split(&self, count: usize) -> (Self, Self) {
        let (prefix, suffix) = self.tok.split_at(count);
        let first = Tokens {
            tok: prefix,
            start: 0,
            end: prefix.len(),
        };
        let second = Tokens {
            tok: suffix,
            start: 0,
            end: suffix.len(),
        };
        (second, first)
    }
}

impl InputLength for Token {
    fn input_len(&self) -> usize {
        1
    }
}

impl<'a> Slice<Range<usize>> for Tokens<'a> {
    fn slice(&self, range: Range<usize>) -> Self {
        Tokens {
            tok: self.tok.slice(range.clone()),
            start: self.start + range.start,
            end: self.start + range.end,
        }
    }
}

impl<'a> Slice<RangeTo<usize>> for Tokens<'a> {
    fn slice(&self, range: RangeTo<usize>) -> Self {
        self.slice(0..range.end)
    }
}

impl<'a> Slice<RangeFrom<usize>> for Tokens<'a> {
    #[inline]
    fn slice(&self, range: RangeFrom<usize>) -> Self {
        self.slice(range.start..self.end - self.start)
    }
}

impl<'a> Slice<RangeFull> for Tokens<'a> {
    #[inline]
    fn slice(&self, _: RangeFull) -> Self {
        Tokens {
            tok: self.tok,
            start: self.start,
            end: self.end,
        }
    }
}

impl<'a> InputIter for Tokens<'a> {
    type Item = &'a Token;
    type Iter = Enumerate<::std::slice::Iter<'a, Token>>;
    type IterElem = ::std::slice::Iter<'a, Token>;

    #[inline]
    fn iter_indices(&self) -> Enumerate<::std::slice::Iter<'a, Token>> {
        self.tok.iter().enumerate()
    }
    #[inline]
    fn iter_elements(&self) -> ::std::slice::Iter<'a, Token> {
        self.tok.iter()
    }
    #[inline]
    fn position<P>(&self, predicate: P) -> Option<usize>
    where
        P: Fn(Self::Item) -> bool,
    {
        self.tok.iter().position(predicate)
    }
    #[inline]
    fn slice_index(&self, count: usize) -> Result<usize, Needed> {
        if self.tok.len() >= count {
            Ok(count)
        } else {
            Err(Needed::Unknown)
        }
    }
}

impl<'a> UnspecializedInput for Tokens<'a> { }
