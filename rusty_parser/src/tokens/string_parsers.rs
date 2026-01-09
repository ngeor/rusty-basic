use rusty_pc::{Many, Map, Parser};

use crate::ParseError;

pub trait CharToStringParser<I> {
    /// Reads as many chars possible from the underlying parser and returns them as a string.
    fn many_to_str(self) -> impl Parser<I, Output = String, Error = ParseError>;

    /// Reads one char possible from the underlying parser and converts it into a string.
    fn one_to_str(self) -> impl Parser<I, Output = String, Error = ParseError>;
}

impl<I, P> CharToStringParser<I> for P
where
    I: Clone,
    P: Parser<I, Output = char, Error = ParseError>,
{
    fn many_to_str(self) -> impl Parser<I, Output = String, Error = ParseError> {
        self.many(String::from, |mut s: String, c| {
            s.push(c);
            s
        })
    }

    fn one_to_str(self) -> impl Parser<I, Output = String, Error = ParseError> {
        self.map(String::from)
    }
}
