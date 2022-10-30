use crate::declaration;
use crate::def_type;
use crate::implementation;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statement;
use crate::types::*;
use crate::user_defined_type;
use rusty_common::*;

pub struct TopLevelTokensParser;

impl TopLevelTokensParser {
    pub fn new() -> Self {
        Self
    }
}

impl Parser for TopLevelTokensParser {
    type Output = ProgramNode;
    fn parse(&self, tokenizer: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        let top_level_token_parser = top_level_token_one_p();
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
                            return Err(QError::SyntaxError(format!("No separator: {}", ch.text)));
                        }
                        tokenizer.unread(ch);
                        let opt_top_level_token = top_level_token_parser.parse_opt(tokenizer)?;
                        match opt_top_level_token {
                            Some(top_level_token) => {
                                top_level_tokens.push(top_level_token);
                                read_separator = false;
                            }
                            _ => {
                                return Err(QError::syntax_error("Expected: top level token"));
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok(top_level_tokens)
    }
}

fn top_level_token_one_p() -> impl Parser<Output = TopLevelTokenNode> {
    Alt5::new(
        def_type::def_type_p().map(TopLevelToken::DefType),
        declaration::declaration_p(),
        implementation::implementation_p(),
        statement::statement_p().map(TopLevelToken::Statement),
        user_defined_type::user_defined_type_p().map(TopLevelToken::UserDefinedType),
    )
    .with_pos()
}
