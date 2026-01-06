use std::collections::HashSet;

use rusty_pc::{ParseErr, ParseResult, Parser, Token, TokenKind};

use crate::ParseError;
use crate::pc_specific::TokenType;

macro_rules! any_token_of {
    (
        MatchMode::$match_mode:ident,
        $($token_type:expr),+$(,)?
    ) => {
        $crate::pc_specific::AnyTokenOf::new_multi(
            $crate::pc_specific::token_parser(),
            &[ $($token_type),+ ],
            false,
            $crate::pc_specific::MatchMode::$match_mode
        )
    };
    (
        $($token_type:expr),+$(,)?
    ) => {
        any_token_of!(MatchMode::Include, $($token_type),+)
    };
}

macro_rules! any_token_of_ws {
    (
        $($token_type:expr),+$(,)?
    ) => {
        $crate::pc_specific::AnyTokenOf::new_multi($crate::pc_specific::token_parser(), &[ $($token_type),+ ], true, $crate::pc_specific::MatchMode::Include)
    };
}

pub(crate) use {any_token_of, any_token_of_ws};

/// Parses a token as long as it's one of the desired token kinds.
/// This is an optimization to reduce the number of types created.
/// It could be achieved with `any_token().filter().with_expected_message()`,
/// without a dedicated `struct`.
pub struct AnyTokenOf<P> {
    /// The parser that provides tokens.
    parser: P,

    /// The token kinds that the parser is looking for.
    token_kinds: HashSet<TokenKind>,

    /// The syntax error message to return for non-fatal errors.
    err_msg: String,

    /// Are leading and trailing whitespaces allowed?
    ws: bool,

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
        err_msg: String,
        ws: bool,
        match_mode: MatchMode,
    ) -> Self {
        Self {
            parser,
            token_kinds,
            err_msg,
            ws,
            match_mode,
        }
    }

    pub fn new_multi(
        parser: P,
        token_types: &[TokenType],
        ws: bool,
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

        Self::new(parser, token_kinds, err_msg, ws, match_mode)
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
        if self.ws {
            self.parse_token_padded_by_ws(input)
        } else {
            self.parse_token(input)
        }
    }
}

impl<P> AnyTokenOf<P> {
    fn parse_token_padded_by_ws<I, C>(&self, input: I) -> ParseResult<I, Token, ParseError>
    where
        I: Clone,
        P: Parser<I, C, Output = Token, Error = ParseError>,
    {
        let original_input = input.clone();

        // parse either the target token or (if self.ws is true) the leading whitespace
        let (input, opt_token) = self.parse_leading_ws_or_token(input)?;

        let (input, token) = match opt_token {
            // we already found the token
            Some(token) => (input, token),
            _ => {
                // we only found the leading whitespace
                // parse the token, and if the error is not fatal, return the original input
                // i.e. undo parsing the leading whitespace
                match self.parse_token(input) {
                    Ok((input, token)) => Ok((input, token)),
                    Err((false, _, err)) => Err((false, original_input, err)),
                    Err(err) => Err(err),
                }?
            }
        };

        // parse the trailing whitespace
        let input = self.parse_trailing_ws(input)?;
        Ok((input, token))
    }

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

    /// Perform the first parse, which can yield either the token we're looking for
    /// or an optional leading whitespace.
    fn parse_leading_ws_or_token<I, C>(&self, input: I) -> ParseResult<I, Option<Token>, ParseError>
    where
        I: Clone,
        P: Parser<I, C, Output = Token, Error = ParseError>,
    {
        let original_input = input.clone();
        match self.parser.parse(input) {
            Ok((input, token)) => {
                if self.accept_token(&token) {
                    // found it
                    Ok((input, Some(token)))
                } else if TokenType::Whitespace.get_index() == token.kind() {
                    // found leading whitespace
                    Ok((input, None))
                } else {
                    self.to_syntax_err(original_input)
                }
            }
            Err((false, _, _)) => self.to_syntax_err(original_input),
            Err(err) => Err(err),
        }
    }

    /// Parse the optional trailing whitespace.
    /// This method returns the input from which the next parser should continue.
    /// If the trailing whitespace was not found, then that is the original input.
    fn parse_trailing_ws<I, C>(&self, input: I) -> Result<I, ParseErr<I, ParseError>>
    where
        I: Clone,
        P: Parser<I, C, Output = Token, Error = ParseError>,
    {
        let original_input = input.clone();
        match self.parser.parse(input) {
            Ok((input, ws_token)) => {
                if ws_token.kind() == TokenType::Whitespace.get_index() {
                    // ok accept trailing whitespace
                    Ok(input)
                } else {
                    Ok(original_input)
                }
            }
            Err((false, _, _)) => Ok(original_input),
            Err(err) => Err(err),
        }
    }

    fn accept_token(&self, token: &Token) -> bool {
        match self.match_mode {
            MatchMode::Include => self.token_kinds.contains(&token.kind()),
            MatchMode::Exclude => !self.token_kinds.contains(&token.kind()),
        }
    }

    /// Creates the syntax error when the desired token kinds are not found.
    fn to_syntax_err<I, O>(&self, input: I) -> ParseResult<I, O, ParseError> {
        Err((false, input, ParseError::syntax_error(&self.err_msg)))
    }
}
