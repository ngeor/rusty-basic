use rusty_pc::{ParseResult, Parser};

use crate::ParseError;

pub trait WithExpected<I>: Parser<I, Error = ParseError>
where
    Self: Sized,
{
    fn with_expected_message<F>(
        self,
        f: F,
    ) -> impl Parser<I, Output = Self::Output, Error = ParseError>
    where
        F: MessageProvider,
    {
        WithExpectedMessage::new(self, f, true)
    }
}

impl<I, P> WithExpected<I> for P where P: Parser<I, Error = ParseError> {}

struct WithExpectedMessage<P, F>(P, F, bool);

impl<P, F> WithExpectedMessage<P, F> {
    pub fn new(parser: P, f: F, allow_incomplete: bool) -> Self {
        Self(parser, f, allow_incomplete)
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
    P: Parser<I, Error = ParseError>,
    F: MessageProvider,
{
    type Output = P::Output;
    type Error = ParseError;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.0.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((false, i, _)) => Err((!self.2, i, ParseError::SyntaxError(self.1.to_str()))),
            Err(err) => Err(err),
        }
    }

    fn no_incomplete(self) -> impl Parser<I, Output = Self::Output, Error = Self::Error>
    where
        Self: Sized,
    {
        Self(self.0, self.1, false)
    }
}
