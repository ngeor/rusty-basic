use crate::pc::{ChainParser, ParseResult};
use crate::ParseError;

// TODO make QError generic param too

/// A parser uses a [Tokenizer] in order to produce a result.
pub trait Parser<I> {
    type Output;

    fn parse(&self, input: I) -> ParseResult<I, Self::Output, ParseError>;

    /**
     * Not reviewed yet
     */

    fn chain<RF, R, F, O>(self, right_factory: RF, combiner: F) -> ChainParser<Self, RF, F>
    where
        Self: Sized,
        RF: Fn(&Self::Output) -> R,
        R: Parser<I>,
        F: Fn(Self::Output, R::Output) -> O,
    {
        ChainParser::new(self, right_factory, combiner)
    }
}
