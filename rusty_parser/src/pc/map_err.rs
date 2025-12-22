use crate::pc::{ParseResult, Parser};
use crate::{parser_declaration, ParseError};

pub trait Errors<I>: Parser<I> {
    fn with_expected_message<F>(self, f: F) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
        F: MessageProvider,
    {
        WithExpectedMessage::new(self, f)
    }

    #[deprecated]
    fn or_fail(self, err: ParseError) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
    {
        OrFailParser::new(self, err)
    }

    #[deprecated]
    fn no_incomplete(self) -> impl Parser<I, Output = Self::Output>
    where
        Self: Sized,
    {
        NoIncompleteParser::new(self)
    }
}

impl<I, P> Errors<I> for P where P: Parser<I> {}

struct OrFailParser<P> {
    parser: P,
    err: ParseError,
}

impl<P> OrFailParser<P> {
    pub fn new(parser: P, err: ParseError) -> Self {
        Self { parser, err }
    }
}

impl<I, P> Parser<I> for OrFailParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((false, i, _)) => Err((true, i, self.err.clone())),
            Err(err) => Err(err),
        }
    }
}

parser_declaration!(struct NoIncompleteParser);

impl<I, P> Parser<I> for NoIncompleteParser<P>
where
    P: Parser<I>,
{
    type Output = P::Output;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.parser.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((_, i, err)) => Err((true, i, err)),
        }
    }
}

struct WithExpectedMessage<P, F>(P, F);

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

impl<I, P, F> Parser<I> for WithExpectedMessage<P, F>
where
    P: Parser<I>,
    F: MessageProvider,
{
    type Output = P::Output;
    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, ParseError> {
        match self.0.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((false, i, _)) => Err((false, i, ParseError::SyntaxError(self.1.to_str()))),
            Err(err) => Err(err),
        }
    }
}
