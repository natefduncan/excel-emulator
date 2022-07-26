#[macro_use] extern crate lalrpop_util;
lalrpop_mod!(pub excel); 

#[cfg(test)]
mod tests {
    use crate::excel::*; 

    #[test]
	fn excel() {
		assert!(TermParser::new().parse("22").is_ok());
		assert!(TermParser::new().parse("(22)").is_ok());
		assert!(TermParser::new().parse("((((22))))").is_ok());
		assert!(TermParser::new().parse("((22)").is_err());
	}
}
