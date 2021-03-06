use crate::parser::pc::{Reader, ReaderResult, Undo};

/// This module holds parsers that modify the result of another parser, together with a function.
use super::Parser;

macro_rules! unary_fn_parser {
    ($name:tt) => {
        pub struct $name<S, F>(S, F);

        impl<S, F> $name<S, F> {
            pub fn new(source: S, f: F) -> Self {
                Self(source, f)
            }
        }
    };
}

// Maps the successful result of a parser.
unary_fn_parser!(Map);

impl<R, S, F, U> Parser<R> for Map<S, F>
where
    R: Reader,
    S: Parser<R>,
    F: Fn(S::Output) -> U,
{
    type Output = U;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, opt_item.map(&self.1)))
    }
}

// Maps the successful result of a parser, optionally rejecting it with an error.
unary_fn_parser!(AndThen);

impl<R, S, F, U> Parser<R> for AndThen<S, F>
where
    R: Reader,
    S: Parser<R>,
    F: Fn(S::Output) -> Result<U, R::Err>,
{
    type Output = U;

    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => match (self.1)(item) {
                Ok(result) => Ok((reader, Some(result))),
                Err(e) => Err((reader, e)),
            },
            _ => Ok((reader, None)),
        }
    }
}

// Validates the result of a parser.
// The validating function can return:
// Ok(true) -> success
// Ok(false) -> undo
// Err -> err
unary_fn_parser!(Validate);

impl<R, S, F> Parser<R> for Validate<S, F>
where
    R: Reader + Undo<S::Output>,
    S: Parser<R>,
    F: Fn(&S::Output) -> Result<bool, R::Err>,
{
    type Output = S::Output;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => match (self.1)(&item) {
                Ok(true) => Ok((reader, Some(item))),
                Ok(false) => Ok((reader.undo(item), None)),
                Err(err) => Err((reader, err)),
            },
            None => Ok((reader, None)),
        }
    }
}

/// Same as OrThrow but the error is not calculated by a function
pub struct OrThrowVal<S, E>(S, E);

impl<S, E> OrThrowVal<S, E> {
    pub fn new(source: S, err: E) -> Self {
        Self(source, err)
    }
}

impl<R, S, E> Parser<R> for OrThrowVal<S, E>
where
    R: Reader<Err = E>,
    S: Parser<R>,
    E: Clone,
{
    type Output = S::Output;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        if opt_item.is_some() {
            Ok((reader, opt_item))
        } else {
            Err((reader, self.1.clone()))
        }
    }
}

// Similar to Filter, but the source parser returns the same item as the reader.
// This is due to the inability to have an Undo in the Reader for the Reader's
// item.
unary_fn_parser!(FilterReaderItem);

impl<R, S, F> Parser<R> for FilterReaderItem<S, F>
where
    R: Reader,
    R::Item: Copy,
    S: Parser<R, Output = R::Item>,
    F: Fn(R::Item) -> bool,
{
    type Output = S::Output;
    fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                if (self.1)(item) {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo_item(item), None))
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

/// Offers chaining methods that result in unary parsers that work with a function.
pub trait UnaryFnParser<R: Reader>: Parser<R> + Sized {
    /// Maps the result of this parser with the given function.
    fn map<F, U>(self, map: F) -> Map<Self, F>
    where
        F: Fn(Self::Output) -> U,
    {
        Map::new(self, map)
    }

    /// Maps the result of this parser with the given function. The function
    /// can reject the parsing result with an error.
    fn and_then<F, U>(self, map: F) -> AndThen<Self, F>
    where
        F: Fn(Self::Output) -> Result<U, R::Err>,
    {
        AndThen::new(self, map)
    }

    /// Validates the result of a parser.
    /// The validating function can return:
    /// - Ok(true) -> success
    /// - Ok(false) -> undo
    /// - Err -> err
    fn validate<F>(self, validation: F) -> Validate<Self, F>
    where
        R: Undo<Self::Output>,
        F: Fn(&Self::Output) -> Result<bool, R::Err>,
    {
        Validate::new(self, validation)
    }

    /// Returns a new parser which filters the result of this parser.
    /// The filtering function has access to a copy of the item.
    /// This parser must return the same item as the reader.
    fn filter_reader_item<F>(self, f: F) -> FilterReaderItem<Self, F>
    where
        F: Fn(Self::Output) -> bool,
        R: Reader<Item = Self::Output>,
        Self::Output: Copy,
    {
        FilterReaderItem::new(self, f)
    }
}

impl<R: Reader, T> UnaryFnParser<R> for T where T: Parser<R> {}
