use std::marker::PhantomData;

use crate::common::*;
use crate::parser::declaration;
use crate::parser::def_type;
use crate::parser::implementation;
use crate::parser::pc::binary::BinaryParser;
use crate::parser::pc::unary::UnaryParser;
use crate::parser::pc::unary_fn::UnaryFnParser;
use crate::parser::pc::{Parser, Reader, ReaderResult};
use crate::parser::statement;
use crate::parser::types::*;
use crate::parser::user_defined_type;

pub struct TopLevelTokensParser<R>(PhantomData<R>);

impl<R> TopLevelTokensParser<R> {
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<R> Parser<R> for TopLevelTokensParser<R>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    type Output = ProgramNode;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        let mut r = reader;
        loop {
            let (tmp, opt_item) = r.read()?;
            r = tmp;
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
                            return Err((r, QError::SyntaxError(format!("No separator: {}", ch))));
                        }
                        let (tmp, opt_top_level_token) =
                            top_level_token_one_p().parse(r.undo_item(ch))?;
                        r = tmp;
                        match opt_top_level_token {
                            Some(top_level_token) => {
                                top_level_tokens.push(top_level_token);
                                read_separator = false;
                            }
                            _ => {
                                return Err((r, QError::syntax_error("Expected: top level token")));
                            }
                        }
                    }
                }
                _ => {
                    break;
                }
            }
        }
        Ok((r, Some(top_level_tokens)))
    }
}

fn top_level_token_one_p<R>() -> impl Parser<R, Output = TopLevelTokenNode>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    def_type::def_type_p()
        .map(TopLevelToken::DefType)
        .or(declaration::declaration_p())
        .or(implementation::implementation_p())
        .or(statement::statement_p().map(TopLevelToken::Statement))
        .or(user_defined_type::user_defined_type_p().map(TopLevelToken::UserDefinedType))
        .with_pos()
}
