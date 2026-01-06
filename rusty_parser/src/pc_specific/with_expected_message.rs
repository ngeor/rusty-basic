use rusty_pc::{ParseResult, Parser, SetContext};

use crate::ParseError;

pub trait WithExpected<I, C>: Parser<I, C, Error = ParseError>
where
    Self: Sized,
{
    fn with_expected_message<F>(
        self,
        f: F,
    ) -> impl Parser<I, C, Output = Self::Output, Error = ParseError>
    where
        F: MessageProvider,
    {
        WithExpectedMessage::new(self, f)
    }
}

impl<I, C, P> WithExpected<I, C> for P where P: Parser<I, C, Error = ParseError> {}

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

impl<I, C, P, F> Parser<I, C> for WithExpectedMessage<P, F>
where
    P: Parser<I, C, Error = ParseError>,
    F: MessageProvider,
{
    type Output = P::Output;
    type Error = ParseError;

    fn parse(&self, tokenizer: I) -> ParseResult<I, Self::Output, Self::Error> {
        match self.0.parse(tokenizer) {
            Ok(value) => Ok(value),
            Err((false, i, _)) => Err((false, i, ParseError::SyntaxError(self.1.to_str()))),
            Err(err) => Err(err),
        }
    }
}

impl<C, P, F> SetContext<C> for WithExpectedMessage<P, F>
where
    P: SetContext<C>,
{
    fn set_context(&mut self, ctx: C) {
        self.0.set_context(ctx);
    }
}
