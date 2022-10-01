use crate::common::*;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::implementation;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statement;
use crate::parser::types::*;
use crate::parser::user_defined_type;
use std::convert::TryFrom;

pub struct TopLevelTokensParser;

impl TopLevelTokensParser {
    pub fn new() -> Self {
        Self
    }
}

impl ParserBase for TopLevelTokensParser {
    type Output = ProgramNode;
}

impl Parser for TopLevelTokensParser {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Self::Output, QError> {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        loop {
            let opt_item = reader.read()?;
            match opt_item {
                Some(ch) => {
                    let token_type = TokenType::try_from(ch.kind)?;
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
                        reader.unread(ch);
                        let opt_top_level_token = top_level_token_one_p().parse_opt(reader)?;
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
