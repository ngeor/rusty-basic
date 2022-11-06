use crate::def_type;
use crate::implementation;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statement;
use crate::types::*;
use crate::user_defined_type;
use crate::{declaration, ParseError};

pub struct ProgramParser;

impl ProgramParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for ProgramParser {
    type Output = Program;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, ParseError> {
        let mut read_separator = true; // we are at the beginning of the file
        let mut program: Program = vec![];
        let global_statement_parser = global_statement_pos_p();
        loop {
            let opt_item = tokenizer.read()?;
            match opt_item {
                Some(ch) => {
                    let token_type = TokenType::from_token(&ch);
                    if token_type == TokenType::Whitespace {
                        // skip whitespace
                    } else if token_type == TokenType::Eol || token_type == TokenType::Colon {
                        read_separator = true;
                    } else {
                        // if it is a comment, we are allowed to read it without a separator
                        let can_read = token_type == TokenType::SingleQuote || read_separator;
                        if !can_read {
                            return Err(ParseError::SyntaxError(format!(
                                "No separator: {}",
                                ch.text
                            )));
                        }
                        tokenizer.unread(ch);
                        let opt_global_statement_pos =
                            global_statement_parser.parse_opt(tokenizer)?;
                        match opt_global_statement_pos {
                            Some(global_statement_pos) => {
                                program.push(global_statement_pos);
                                read_separator = false;
                            }
                            _ => {
                                return Err(ParseError::syntax_error("Expected: top level token"));
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok(program)
    }
}

fn global_statement_pos_p() -> impl Parser<Output = GlobalStatementPos> {
    Alt5::new(
        def_type::def_type_p().map(GlobalStatement::DefType),
        declaration::declaration_p(),
        implementation::implementation_p(),
        statement::statement_p().map(GlobalStatement::Statement),
        user_defined_type::user_defined_type_p().map(GlobalStatement::UserDefinedType),
    )
    .with_pos()
}
