/// This module holds parsers that combine two other parsers in order to form
/// their result.
use super::{Parser, Reader, ReaderResult, Undo};

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

// OptLeftAndRight requires that the right parser returns a result.
// It will undo the first result if it was `Some` and the second was `None`.
binary_parser!(OptLeftAndRight);

impl<A, B, R> Parser<R> for OptLeftAndRight<A, B>
where
    R: Reader + Undo<A::Output>,
    A: Parser<R>,
    B: Parser<R>,
{
    type Output = (Option<A::Output>, B::Output);

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
        let (reader, opt_a) = self.0.parse(reader)?;
        let (reader, opt_b) = self.1.parse(reader)?;
        match opt_b {
            Some(b) => Ok((reader, Some((opt_a, b)))),
            _ => match opt_a {
                Some(a) => Ok((reader.undo(a), None)),
                _ => Ok((reader, None)),
            },
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

/// Offers chaining methods that result in binary parsers.
pub trait BinaryParser<R: Reader>: Parser<R> {
    /// Returns a new parser that combines the result of the current parser
    /// with the given parser. Both parsers must return a result. If the
    /// second parser returns `None`, the first result will be undone.
    fn and<B>(self, other: B) -> LeftAndRight<Self, B>
    where
        R: Undo<Self::Output>,
        B: Parser<R>,
    {
        LeftAndRight::new(self, other)
    }

    /// Returns a new parser that combines the result of the current parser
    /// with the given parser. The current parser must return a value,
    /// but the given parser can return `None`.
    fn and_opt<B>(self, other: B) -> LeftAndOptRight<Self, B>
    where
        B: Parser<R>,
    {
        LeftAndOptRight::new(self, other)
    }

    /// Returns a new parser prepending the given parser before the current one.
    /// The given parser can return `None`. Its result will be undone if the
    /// current parser returns `None`.
    fn preceded_by<B>(self, other: B) -> OptLeftAndRight<B, Self>
    where
        B: Parser<R>,
        R: Undo<B::Output>,
    {
        OptLeftAndRight::new(other, self)
    }

    /// Returns a new parser which returns the result of the given parser,
    /// as long as the second parser returns `None`.
    ///
    /// If the given parser succeeds, its result will be undone as well as the
    /// result of the current parser.
    fn unless_followed_by<B>(self, other: B) -> RollbackLeftIfRight<Self, B>
    where
        R: Undo<Self::Output> + Undo<B::Output>,
        B: Parser<R>,
    {
        RollbackLeftIfRight::new(self, other)
    }

    /// Returns a parser which combines the results of this parser and the given one.
    /// If the given parser fails, the parser will panic.
    fn and_demand<B>(self, other: B) -> LeftAndDemandRight<Self, B>
    where
        B: Parser<R>,
    {
        LeftAndDemandRight::new(self, other)
    }
}

impl<R: Reader, T> BinaryParser<R> for T where T: Parser<R> {}
