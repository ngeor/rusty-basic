use crate::parser::pc::{Reader, ReaderResult, Undo};

/// This module holds parsers that combine two other parsers in order to form
/// their result.
use super::Parser;

macro_rules! binary_parser {
    ($name:tt) => {
        pub struct $name<A, B>(A, B);

        impl<A, B> $name<A, B> {
            pub fn new(a: A, b: B) -> Self {
                Self(a, b)
            }
        }
    };
}

macro_rules! binary_dyn_parser {
    ($name:tt) => {
        pub struct $name<R, A, B>
        where
            R: Reader,
        {
            left: Box<dyn Parser<R, Output = A>>,
            right: Box<dyn Parser<R, Output = B>>,
        }

        impl<R, A, B> $name<R, A, B>
        where
            R: Reader,
        {
            pub fn new(
                left: Box<dyn Parser<R, Output = A>>,
                right: Box<dyn Parser<R, Output = B>>,
            ) -> Self {
                Self { left, right }
            }

            pub fn new_from_unboxed<T, U>(left: T, right: U) -> Self
            where
                T: Parser<R, Output = A> + 'static,
                U: Parser<R, Output = B> + 'static,
            {
                Self::new(Box::new(left), Box::new(right))
            }
        }
    };
}

// LeftAndRight requires that both left and right parsers return a result.
// It will undo the first result if the second is `None`.
binary_parser!(LeftAndRight);

impl<R, A, B> Parser<R> for LeftAndRight<A, B>
where
    R: Reader + Undo<A::Output>,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                match opt_b {
                    Some(b) => Ok((reader, Some((a, b)))),
                    None => Ok((reader.undo(a), None)),
                }
            }
            None => Ok((reader, None)),
        }
    }
}

// LeftAndOptRight requires that the left parser returns a result.
binary_parser!(LeftAndOptRight);

impl<R, A, B> Parser<R> for LeftAndOptRight<A, B>
where
    R: Reader,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (A::Output, Option<B::Output>);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                Ok((reader, Some((a, opt_b))))
            }
            None => Ok((reader, None)),
        }
    }
}

/// Combines the result of the first parser with the result of the parser
/// constructed by the second function. The function has access to the first
/// parser's result.
///
/// The resulting parser succeeds if the first result is `Ok(Some)`.
pub struct LeftAndOptRightFactory<A, F>(A, F);

impl<R, A, F, B> Parser<R> for LeftAndOptRightFactory<A, F>
where
    R: Reader,
    A: Parser<R>,
    F: Fn(&A::Output) -> B,
    B: Parser<R>,
{
    type Output = (A::Output, Option<B::Output>);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let next_parser = (self.1)(&a);
                let (reader, opt_b) = next_parser.parse(reader)?;
                Ok((reader, Some((a, opt_b))))
            }
            _ => Ok((reader, None)),
        }
    }
}

// Returns the result of the left parser, unless the right parser also succeeds.
binary_parser!(RollbackLeftIfRight);

impl<R, A, B> Parser<R> for RollbackLeftIfRight<A, B>
where
    R: Reader + Undo<A::Output> + Undo<B::Output>,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = A::Output;
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                match opt_b {
                    Some(b) => Ok((reader.undo(b).undo(a), None)),
                    None => Ok((reader, Some(a))),
                }
            }
            None => Ok((reader, None)),
        }
    }
}

// Similar to And, but without undo.
binary_parser!(LeftAndDemandRight);

impl<R, A, B> Parser<R> for LeftAndDemandRight<A, B>
where
    R: Reader,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (A::Output, B::Output);
    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        match opt_a {
            Some(a) => {
                let (reader, opt_b) = self.1.parse(reader)?;
                Ok((
                    reader,
                    Some((a, opt_b.expect("Right parser of LeftAndDemandRight failed"))),
                ))
            }
            _ => Ok((reader, None)),
        }
    }
}

// Implements the "or" parser, returning the first successful
// result out of the two given parsers.
pub struct LeftOrRight<R, T> {
    left: Box<dyn Parser<R, Output = T>>,
    right: Box<dyn Parser<R, Output = T>>,
}

impl<R, T> Parser<R> for LeftOrRight<R, T>
where
    R: Reader,
{
    type Output = T;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.left.parse(reader)?;
        match opt_a {
            Some(a) => Ok((reader, Some(a))),
            _ => self.right.parse(reader),
        }
    }
}

/// Offers chaining methods that result in binary parsers.
pub trait BinaryParser<R: Reader>: Parser<R> + Sized {
    /// Returns a new parser that combines the result of the current parser
    /// with the given parser. Both parsers must return a result. If the
    /// second parser returns `None`, the first result will be undone.
    fn and<B>(self, other: B) -> LeftAndRight<Self, B>
    where
        R: Undo<Self::Output>,
        B: Sized + Parser<R>,
    {
        LeftAndRight::new(self, other)
    }

    /// Returns a new parser that combines the result of the current parser
    /// with the given parser. The current parser must return a value,
    /// but the given parser can return `None`.
    fn and_opt<B>(self, other: B) -> LeftAndOptRight<Self, B>
    where
        B: Sized + Parser<R>,
    {
        LeftAndOptRight::new(self, other)
    }

    /// Combines the result of the first parser with the result of the parser
    /// constructed by the second function. The function has access to the first
    /// parser's result.
    ///
    /// The resulting parser succeeds if the first result is `Ok(Some)`.
    fn and_opt_factory<F, B>(self, factory: F) -> LeftAndOptRightFactory<Self, F>
    where
        F: Fn(&Self::Output) -> B,
        B: Sized + Parser<R>,
    {
        LeftAndOptRightFactory(self, factory)
    }

    /// Returns a new parser which returns the result of the given parser,
    /// as long as the second parser returns `None`.
    ///
    /// If the given parser succeeds, its result will be undone as well as the
    /// result of the current parser.
    fn unless_followed_by<B>(self, other: B) -> RollbackLeftIfRight<Self, B>
    where
        R: Undo<Self::Output> + Undo<B::Output>,
        B: Sized + Parser<R>,
    {
        RollbackLeftIfRight::new(self, other)
    }

    /// Returns a parser which combines the results of this parser and the given one.
    /// If the given parser fails, the parser will panic.
    fn and_demand<B>(self, other: B) -> LeftAndDemandRight<Self, B>
    where
        B: Sized + Parser<R>,
    {
        LeftAndDemandRight::new(self, other)
    }

    /// Returns a parser which will return the result of this parser if it
    /// is successful, otherwise it will use the given parser.
    fn or<B>(self, other: B) -> LeftOrRight<R, Self::Output>
    where
        Self: 'static,
        B: Sized + Parser<R, Output = Self::Output> + 'static,
    {
        LeftOrRight {
            left: Box::new(self),
            right: Box::new(other),
        }
    }
}

impl<R: Reader, T> BinaryParser<R> for T where T: Parser<R> {}
