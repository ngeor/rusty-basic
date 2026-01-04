use std::collections::HashSet;

use rusty_pc::{Parser, Token, TokenKind};

use crate::ParseError;
use crate::pc_specific::TokenType;

macro_rules! any_token_of {
    (
        $($token_type:expr),+$(,)?
    ) => {
        $crate::pc_specific::AnyTokenOf::new_multi($crate::pc_specific::token_parser(), &[ $($token_type),+ ])
    };
}

pub(crate) use any_token_of;

/// Parses a token as long as it's one of the desired token kinds.
/// This is an optimization to reduce the number of types created.
/// It could be achieved with `any_token().filter().with_expected_message()`,
/// without a dedicated `struct`.
pub struct AnyTokenOf<P> {
    parser: P,
    token_kinds: HashSet<TokenKind>,
    err_msg: String,
}

impl<P> AnyTokenOf<P> {
    pub fn new(parser: P, token_kinds: HashSet<TokenKind>, err_msg: String) -> Self {
        Self {
            parser,
            token_kinds,
            err_msg,
        }
    }

    pub fn new_multi(parser: P, token_types: &[TokenType]) -> Self {
        let mut token_kinds: HashSet<TokenKind> = HashSet::new();
        let mut err_msg: String = "Expected: ".to_owned();
        let mut is_first = true;
        for token_type in token_types {
            token_kinds.insert(token_type.get_index());

            if is_first {
                is_first = false;
            } else {
                err_msg.push_str(" or ");
            }

            err_msg.push_str(&token_type.to_string());
        }

        Self::new(parser, token_kinds, err_msg)
    }
}

impl<I, C, P> Parser<I, C> for AnyTokenOf<P>
where
    I: Clone,
    P: Parser<I, C, Output = Token, Error = ParseError>,
{
    type Output = Token;
    type Error = ParseError;

    fn parse(&self, input: I) -> rusty_pc::ParseResult<I, Self::Output, Self::Error> {
        match self.parser.parse(input.clone()) {
            Ok((remaining, token)) => {
                if self.token_kinds.contains(&token.kind()) {
                    Ok((remaining, token))
                } else {
                    Err((false, input, ParseError::syntax_error(&self.err_msg)))
                }
            }
            Err((false, input, _)) => Err((false, input, ParseError::syntax_error(&self.err_msg))),
            Err(err) => Err(err),
        }
    }

    fn set_context(&mut self, _ctx: C) {
        // do nothing
    }
}
