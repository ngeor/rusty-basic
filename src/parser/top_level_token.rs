use crate::common::*;
use crate::parser::base::parsers::{FnMapTrait, HasOutput, OrTrait, Parser};
use crate::parser::base::tokenizers::Tokenizer;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::implementation;
use crate::parser::specific::with_pos::WithPosTrait;
use crate::parser::statement;
use crate::parser::types::*;
use crate::parser::user_defined_type;

pub struct TopLevelTokensParser;

impl TopLevelTokensParser {
    pub fn new() -> Self {
        Self
    }
}

impl HasOutput for TopLevelTokensParser {
    type Output = ProgramNode;
}

impl Parser for TopLevelTokensParser {
    fn parse(&self, reader: &mut impl Tokenizer) -> Result<Option<Self::Output>, QError> {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        loop {
            let opt_item = reader.read()?;
            match opt_item {
                Some(ch) => {
                    if ch == ' ' {
                        // skip whitespace
                    } else if ch == '\r' || ch == '\n' || ch == ':' {
                        read_separator = true;
                    } else {
                        // if it is a comment, we are allowed to read it without a separator
                        let can_read = ch == '\'' || read_separator;
                        if !can_read {
                            return Err(QError::SyntaxError(format!("No separator: {}", ch)));
                        }
                        reader.unread(ch);
                        let opt_top_level_token = top_level_token_one_p().parse(reader)?;
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
        Ok(Some(top_level_tokens))
    }
}

fn top_level_token_one_p() -> impl Parser<Output = TopLevelTokenNode> {
    def_type::def_type_p()
        .fn_map(TopLevelToken::DefType)
        .or(declaration::declaration_p())
        .or(implementation::implementation_p())
        .or(statement::statement_p().fn_map(TopLevelToken::Statement))
        .or(user_defined_type::user_defined_type_p().fn_map(TopLevelToken::UserDefinedType))
        .with_pos()
}
