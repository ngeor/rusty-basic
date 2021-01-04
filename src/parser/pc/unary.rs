use std::convert::TryFrom;
use std::marker::PhantomData;

use crate::common::{AtLocation, HasLocation, Locatable};
use crate::parser::pc::{Reader, ReaderResult, Undo};

/// This module holds parsers that modify the result of another parser.
use super::Parser;

macro_rules! unary_parser {
    ($name:tt) => {
        pub struct $name<S>(S);

        impl<S> $name<S> {
            pub fn new(source: S) -> Self {
                Self(source)
            }
        }
    };
}

// Maps `None` to `Some(default)`.
unary_parser!(MapNoneToDefault);

impl<R, S> Parser<R> for MapNoneToDefault<S>
where
    R: Reader,
    S: Parser<R>,
    S::Output: Default,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, Some(opt_item.unwrap_or_default())))
    }
}

// Accepts `None` results by wrapping them in an `Option`.
unary_parser!(Optional);

impl<R, S> Parser<R> for Optional<S>
where
    R: Reader,
    S: Parser<R>,
{
    type Output = Option<S::Output>;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, Some(opt_item)))
    }
}

// Keeps the left side of a tuple.
unary_parser!(KeepLeft);

impl<R, S, T, U> Parser<R> for KeepLeft<S>
where
    R: Reader,
    S: Parser<R, Output = (T, U)>,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|(t, _)| t);
        Ok((reader, mapped_opt_item))
    }
}

// Keeps the right side of a tuple.
unary_parser!(KeepRight);

impl<R, S, T, U> Parser<R> for KeepRight<S>
where
    R: Reader,
    S: Parser<R, Output = (T, U)>,
{
    type Output = U;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|(_, u)| u);
        Ok((reader, mapped_opt_item))
    }
}

// Keeps the middle of a tuple.
unary_parser!(KeepMiddle);

impl<R, S, A, B, C> Parser<R> for KeepMiddle<S>
where
    R: Reader,
    S: Parser<R, Output = ((A, B), C)>,
{
    type Output = B;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        let mapped_opt_item = opt_item.map(|((_, b), _)| b);
        Ok((reader, mapped_opt_item))
    }
}

// Peeks the result by undoing it.
unary_parser!(PeekReaderItem);

impl<R, S> Parser<R> for PeekReaderItem<S>
where
    R: Reader,
    R::Item: Copy,
    S: Parser<R, Output = R::Item>,
{
    type Output = S::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => Ok((reader.undo_item(item), Some(item))),
            _ => Ok((reader, None)),
        }
    }
}

// Adds location information to the result of a parser.
unary_parser!(WithPos);

impl<S, R> Parser<R> for WithPos<S>
where
    R: Reader + HasLocation,
    S: Parser<R>,
{
    type Output = Locatable<S::Output>;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let pos = reader.pos();
        let (reader, opt_item) = self.0.parse(reader)?;
        Ok((reader, opt_item.map(|item| item.at(pos))))
    }
}

// Negates the result of the underlying parser.
// If the parser succeeds, the item is undone and `None` is returned.
// If the parser fails, a successful empty string is returned.
unary_parser!(Negate);

impl<R, S> Parser<R> for Negate<S>
where
    R: Reader<Item = char> + Undo<S::Output>,
    S: Parser<R>,
{
    type Output = String;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => {
                // found the exit
                Ok((reader.undo(item), None))
            }
            _ => Ok((reader, Some(String::new()))),
        }
    }
}

/// Converts with the TryFrom trait.
pub struct TryFromParser<S, T>(S, PhantomData<T>);

impl<S, T> TryFromParser<S, T> {
    pub fn new(source: S) -> Self {
        Self(source, PhantomData)
    }
}

impl<R, S, T> Parser<R> for TryFromParser<S, T>
where
    R: Reader + Undo<S::Output>,
    S: Parser<R>,
    S::Output: Copy,
    T: TryFrom<S::Output>,
{
    type Output = T;
    fn parse(&self, reader: R) -> ReaderResult<R, T, R::Err> {
        let (reader, opt_item) = self.0.parse(reader)?;
        match opt_item {
            Some(item) => match T::try_from(item) {
                Ok(t) => Ok((reader, Some(t))),
                _ => Ok((reader.undo(item), None)),
            },
            _ => Ok((reader, None)),
        }
    }
}

/// Offers chaining methods that result in unary parsers.
pub trait UnaryParser<R: Reader>: Parser<R> + Sized {
    /// Maps an unsuccessful result of the given parser into a successful default value.
    fn map_none_to_default(self) -> MapNoneToDefault<Self>
    where
        Self::Output: Default,
    {
        MapNoneToDefault::new(self)
    }

    /// Accepts unsuccessful results by wrapping the results of the given parser
    /// into an `Option`.
    fn optional(self) -> Optional<Self> {
        Optional::new(self)
    }

    /// Keeps the left part of a tuple.
    fn keep_left<T, U>(self) -> KeepLeft<Self>
    where
        Self: Parser<R, Output = (T, U)>,
    {
        KeepLeft::new(self)
    }

    /// Keeps the right part of a tuple.
    fn keep_right<T, U>(self) -> KeepRight<Self>
    where
        Self: Parser<R, Output = (T, U)>,
    {
        KeepRight::new(self)
    }

    /// Keeps the middle part of a tuple.
    fn keep_middle<A, B, C>(self) -> KeepMiddle<Self>
    where
        Self: Parser<R, Output = ((A, B), C)>,
    {
        KeepMiddle::new(self)
    }

    /// Peeks the item of the underlying reader by undoing it.
    fn peek_reader_item(self) -> PeekReaderItem<Self>
    where
        Self: Parser<R, Output = R::Item>,
        R::Item: Copy,
    {
        PeekReaderItem::new(self)
    }

    /// Adds location information to the result of this parser.
    fn with_pos(self) -> WithPos<Self> {
        WithPos::new(self)
    }

    /// Converts the result of the parser with the `TryFrom` trait.
    /// If the conversion fails, the item is undone.
    fn try_from<T>(self) -> TryFromParser<Self, T>
    where
        T: TryFrom<Self::Output>,
        R: Undo<Self::Output>,
    {
        TryFromParser::<Self, T>::new(self)
    }

    /// Negates the result of the underlying parser.
    /// If the parser succeeds, the item is undone and `None` is returned.
    /// If the parser fails, a successful empty string is returned.
    fn negate(self) -> Negate<Self>
    where
        R: Undo<Self::Output>,
    {
        Negate::new(self)
    }
}

impl<R: Reader, T> UnaryParser<R> for T where T: Parser<R> {}