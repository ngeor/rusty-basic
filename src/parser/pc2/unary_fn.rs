/// This module holds parsers that modify the result of another parser, together with a function.
use super::{Parser, Reader, ReaderResult, Undo};

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
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, opt_item.map(&self.1)))
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
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
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

// Throws an error if the parser returns `None`.

unary_fn_parser!(OrThrow);

impl<R, S, F> Parser<R> for OrThrow<S, F>
where
    R: Reader,
    S: Parser<R>,
    F: Fn() -> R::Err,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        if opt_item.is_some() {
            Ok((reader, opt_item))
        } else {
            Err((reader, (self.1)()))
        }
    }
}

// Same as OrThrow but the error is not calculated by a function

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
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        if opt_item.is_some() {
            Ok((reader, opt_item))
        } else {
            Err((reader, self.1.clone()))
        }
    }
}

// Filters the parser result given a predicate. The predicate has access to a
// reference of the item.

unary_fn_parser!(FilterRef);

impl<R, S, F> Parser<R> for FilterRef<S, F>
where
    R: Reader + Undo<S::Output>,
    S: Parser<R>,
    F: Fn(&S::Output) -> bool,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                if (self.1)(&item) {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo(item), None))
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

// Filters the parser result given a predicate. The predicate has access to a
// copy of the item.

unary_fn_parser!(Filter);

impl<R, S, F> Parser<R> for Filter<S, F>
where
    R: Reader + Undo<S::Output>,
    S: Parser<R>,
    S::Output: Copy,
    F: Fn(S::Output) -> bool,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                if (self.1)(item) {
                    Ok((reader, Some(item)))
                } else {
                    Ok((reader.undo(item), None))
                }
            }
            _ => Ok((reader, None)),
        }
    }
}

unary_fn_parser!(FilterReaderItem);

impl<R, S, F> Parser<R> for FilterReaderItem<S, F>
where
    R: Reader,
    R::Item: Copy,
    S: Parser<R, Output = R::Item>,
    F: Fn(R::Item) -> bool,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
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
