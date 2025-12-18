use crate::pc::{ParseResult, Parser, Tokenizer};
use crate::{parser_declaration, ParseError};

pub struct OrFailParser<P> {
    parser: P,
    err: ParseError,
}

impl<P> OrFailParser<P> {
    pub fn new(parser: P, err: ParseError) -> Self {
        Self { parser, err }
    }
}

impl<I: Tokenizer + 'static, P> Parser<I> for OrFailParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None | ParseResult::Expected(_) => ParseResult::Err(self.err.clone()),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}

parser_declaration!(pub struct NoIncompleteParser);

impl<I: Tokenizer + 'static, P> Parser<I> for NoIncompleteParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None => ParseResult::Err(ParseError::syntax_error("Could not parse")),
            ParseResult::Expected(s) => ParseResult::Err(ParseError::SyntaxError(s)),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}

pub struct WithExpectedMessage<P, F>(P, F);

impl<P, F> WithExpectedMessage<P, F> {
    pub fn new(parser: P, f: F) -> Self {
        Self(parser, f)
    }
}

pub trait MessageProvider {
    fn to_str(&self) -> String;
}

impl MessageProvider for &str {
    fn to_str(&self) -> String {
        self.to_string()
    }
}

impl MessageProvider for String {
    fn to_str(&self) -> String {
        self.clone()
    }
}

impl<F> MessageProvider for F
where
    F: Fn() -> String,
{
    fn to_str(&self) -> String {
        (self)()
    }
}

impl<I: Tokenizer + 'static, P, F> Parser<I> for WithExpectedMessage<P, F>
where
    P: Parser<I>,
    F: MessageProvider,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: &mut I) -> ParseResult<Self::Output, ParseError> {
        match self.0.parse(tokenizer) {
            ParseResult::Ok(value) => ParseResult::Ok(value),
            ParseResult::None | ParseResult::Expected(_) => ParseResult::Expected(self.1.to_str()),
            ParseResult::Err(err) => ParseResult::Err(err),
        }
    }
}
