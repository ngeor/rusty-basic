use rusty_pc::{MapErr, Parser};

use crate::ParserError;

pub trait WithExpected<I, C>: Parser<I, C, Error = ParserError>
where
    Self: Sized,
{
    fn with_expected_message<F>(
        self,
        f: F,
    ) -> impl Parser<I, C, Output = Self::Output, Error = ParserError>
    where
        F: MessageProvider,
    {
        self.map_non_fatal_err(move |_| ParserError::Expected(f.to_str()))
    }
}

impl<I, C, P> WithExpected<I, C> for P where P: Parser<I, C, Error = ParserError> {}

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
