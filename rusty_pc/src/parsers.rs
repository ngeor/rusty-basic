use crate::{FilterParser, FilterPredicate};

/// A parser uses the given input in order to produce a result.
pub trait Parser<I, C = ()>
where
    I: InputTrait,
{
    type Output;
    type Error: ParserErrorTrait;

    /// Parses the given input and returns a result.
    fn parse(&mut self, input: &mut I) -> Result<Self::Output, Self::Error>;

    fn filter<F>(self, predicate: F) -> FilterParser<Self, F>
    where
        Self: Sized,
        F: FilterPredicate<Self::Output, Self::Error>,
    {
        FilterParser::new(self, predicate)
    }
}

pub trait InputTrait {
    type Output;

    /// Returns the next character.
    /// Does not advance the position.
    fn peek(&self) -> Self::Output;

    /// Returns the next character.
    /// Advances the position.
    fn read(&mut self) -> Self::Output;

    /// Gets the current position within the source.
    fn get_position(&self) -> usize;

    /// Increase the current position by the given amount.
    fn inc_position_by(&mut self, amount: usize);

    /// Is the input at the end of file.
    fn is_eof(&self) -> bool;

    /// Sets the current position within the source.
    fn set_position(&mut self, position: usize);

    /// Increase the current position by one character.
    fn inc_position(&mut self) {
        self.inc_position_by(1)
    }
}

pub trait ParserErrorTrait: Clone + Default {
    /// Gets a value indicating whether this is a fatal error or not.
    /// Returns true if the error is fatal, false is the error is soft.
    fn is_fatal(&self) -> bool;

    /// Gets a value indicating whether this is a soft error or not.
    /// Returns true if the error is soft, false is the error is fatal.
    fn is_soft(&self) -> bool {
        !self.is_fatal()
    }

    /// Converts this error into a fatal.
    fn to_fatal(self) -> Self;
}
