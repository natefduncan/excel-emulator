#[derive(PartialEq, Debug, Clone)]
pub enum Expr {
    Ident(Ident),
    Lit(Literal),
    Prefix(Prefix, Box<Expr>),
    Infix(Infix, Box<Expr>, Box<Expr>),
	Func {
        name: String, 
        args: Vec<Box<Expr>>
    },
    Reference {
        sheet: String, 
        reference: String 
    }, 
	Array(Vec<Box<Expr>>),
    Error(Error)
}

#[derive(Debug, Clone)]
pub enum Error {
    Null,
    Div, 
    Value,
    Ref, 
    Name, 
    Num, 
    NA, 
    GettingData
}

#[derive(PartialEq, Debug, Clone)]
pub enum Literal {
    Number(f64),
    Bool(bool),
    Text(String),
}

#[derive(PartialEq, Debug, Clone)]
pub enum Prefix {
    Plus,
    Minus,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Infix {
    Plus,
    Minus,
    Divide,
    Multiply,
    Equal,
    NotEqual,
    GreaterThanEqual,
    LessThanEqual,
    GreaterThan,
    LessThan,
}
