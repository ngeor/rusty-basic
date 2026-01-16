use std::collections::HashSet;

use rusty_pc::{ParseResult, Parser, Token, TokenKind};

use crate::ParseError;
use crate::tokens::TokenType;

macro_rules! any_token_of {
    // full options
    (
        types = $($token_type:expr),*
        ;
        symbols = $($symbol:literal),*
        ;
        mode = $match_mode:expr
        $(,)?
    ) => {
        $crate::tokens::AnyTokenOf::new_multi(
            $crate::tokens::any_token(),
            &[ $($token_type),* ],
            &[$($symbol),*],
            $match_mode
        )
    };

    // minus mode
    (
        $($token_type:expr),+
        ;
        symbols = $($symbol:literal),+
        $(,)?
    ) => {
        any_token_of!(
            types = $($token_type),+
            ;
            symbols = $($symbol),+
            ;
            mode = $crate::tokens::MatchMode::Include
        )
    };

    // only types and mode
    (
        $($token_type:expr),+
        ;
        mode = $match_mode:expr
        $(,)?
    ) => {
        any_token_of!(
            types = $($token_type),+
            ;
            symbols =
            ;
            mode = $match_mode
        )
    };

    // only token types
    (
        $($token_type:expr),+$(,)?
    ) => {
        any_token_of!(
            types = $($token_type),+
            ;
            symbols =
            ;
            mode = $crate::tokens::MatchMode::Include
        )
    };
}

macro_rules! any_symbol_of {
    (
        $($symbol:literal),+$(,)?
    ) => {
        any_token_of!(
            types =
            ;
            symbols = $($symbol),+
            ;
            mode = $crate::tokens::MatchMode::Include
        )
    };
}

macro_rules! any_symbol_of_ws {
    (
        $($symbol:literal),+$(,)?
    ) => {
        any_token_of!(
            types =
            ;
            symbols = $($symbol),+
            ;
            mode = $crate::tokens::MatchMode::Include
        ).padded_by_ws()
    };
}

pub(crate) use {any_symbol_of, any_symbol_of_ws, any_token_of};

/// Parses a token as long as it's one of the desired token kinds.
/// This is an optimization to reduce the number of types created.
/// It could be achieved with `any_token().filter().with_expected_message()`,
/// without a dedicated `struct`.
pub struct AnyTokenOf<P> {
    /// The parser that provides tokens.
    parser: P,

    /// The token kinds that the parser is looking for.
    token_kinds: HashSet<TokenKind>,

    /// The symbols that the parser is looking for.
    symbols: HashSet<char>,

    /// The syntax error message to return for non-fatal errors.
    err_msg: String,

    match_mode: MatchMode,
}

#[derive(Default)]
pub enum MatchMode {
    #[default]
    Include,
    Exclude,
}

impl<P> AnyTokenOf<P> {
    pub fn new(
        parser: P,
        token_kinds: HashSet<TokenKind>,
        symbols: HashSet<char>,
        err_msg: String,
        match_mode: MatchMode,
    ) -> Self {
        Self {
            parser,
            token_kinds,
            symbols,
            err_msg,
            match_mode,
        }
    }

    pub fn new_multi(
        parser: P,
        token_types: &[TokenType],
        symbols: &[char],
        match_mode: MatchMode,
    ) -> Self {
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

        for symbol in symbols {
            if is_first {
                is_first = false;
            } else {
                err_msg.push_str(" or ");
            }

            err_msg.push(*symbol);
        }

        Self::new(
            parser,
            token_kinds,
            symbols.iter().copied().collect(),
            err_msg,
            match_mode,
        )
    }
}

impl<I, C, P> Parser<I, C> for AnyTokenOf<P>
where
    I: Clone,
    P: Parser<I, C, Output = Token, Error = ParseError>,
{
    type Output = Token;
    type Error = ParseError;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, Self::Error> {
        self.parse_token(input)
    }
}

impl<P> AnyTokenOf<P> {
    /// Parses the token we're looking for.
    fn parse_token<I, C>(&self, input: I) -> ParseResult<I, Token, ParseError>
    where
        I: Clone,
        P: Parser<I, C, Output = Token, Error = ParseError>,
    {
        let original_input = input.clone();
        match self.parser.parse(input) {
            Ok((input, token)) => {
                if self.accept_token(&token) {
                    // found it
                    Ok((input, token))
                } else {
                    self.to_syntax_err(original_input)
                }
            }
            Err((false, _, _)) => self.to_syntax_err(original_input),
            Err(err) => Err(err),
        }
    }

    fn accept_token(&self, token: &Token) -> bool {
        match self.match_mode {
            MatchMode::Include => self.test_token(token),
            MatchMode::Exclude => !self.test_token(token),
        }
    }

    fn test_token(&self, token: &Token) -> bool {
        self.token_kinds.contains(&token.kind()) || self.test_char(token)
    }

    fn test_char(&self, token: &Token) -> bool {
        if token.kind() == TokenType::Symbol.get_index() {
            let ch = token.as_char();
            self.symbols.contains(&ch)
        } else {
            false
        }
    }

    /// Creates the syntax error when the desired token kinds are not found.
    fn to_syntax_err<I, O>(&self, input: I) -> ParseResult<I, O, ParseError> {
        Err((false, input, ParseError::syntax_error(&self.err_msg)))
    }
}
