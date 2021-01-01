/// This module has parsers that return one or more items.
use super::{Parser, Reader, ReaderResult};
use crate::parser::pc2::unary::{MapNoneToDefault, UnaryParser};

/// Calls the underlying parser multiple times and collects the results in a
/// `Vec`.
pub struct OneOrMore<S>(S);

impl<R, S> Parser<R> for OneOrMore<S>
where
    R: Reader,
    S: Parser<R>,
{
    type Output = Vec<S::Output>;

    fn parse(&self, r: R) -> ReaderResult<R, Self::Output, R::Err> {
        let mut reader = r;
        let mut has_more = true;
        let mut result: Vec<S::Output> = vec![];
        while has_more {
            let (tmp, opt_item) = self.0.parse(reader)?;
            reader = tmp;
            match opt_item {
                Some(item) => {
                    result.push(item);
                }
                _ => {
                    has_more = false;
                }
            }
        }
        if result.is_empty() {
            Ok((reader, None))
        } else {
            Ok((reader, Some(result)))
        }
    }
}

/// One or more items separated by a delimiter.
///
/// - S: The parser the provides the items.
/// - D: The parser that provides the delimiters.
/// - E: An error in case of trailing delimiters.
pub struct OneOrMoreDelimited<S, D, E>(S, D, E);

impl<R, S, D, E> Parser<R> for OneOrMoreDelimited<S, D, E>
where
    R: Reader<Err = E>,
    S: Parser<R>,
    D: Parser<R>,
    E: Clone,
{
    type Output = Vec<S::Output>;

    fn parse(&self, r: R) -> ReaderResult<R, Self::Output, R::Err> {
        let mut reader = r;
        let mut has_more = true;
        let mut read_delimiter = false;
        let mut result: Vec<S::Output> = vec![];
        while has_more {
            let (tmp, opt_item) = self.0.parse(reader)?;
            reader = tmp;
            match opt_item {
                Some(item) => {
                    result.push(item);
                    // scan for delimiter
                    let (tmp, opt_delimiter) = self.1.parse(reader)?;
                    reader = tmp;
                    if opt_delimiter.is_some() {
                        // flag it so we can detect trailing delimiters
                        read_delimiter = true;
                    } else {
                        // exit the loop
                        has_more = false;
                    }
                }
                _ => {
                    if read_delimiter {
                        // error: trailing delimiter
                        return Err((reader, self.2.clone()));
                    } else {
                        // break the loop
                        has_more = false;
                    }
                }
            }
        }
        if result.is_empty() {
            Ok((reader, None))
        } else {
            Ok((reader, Some(result)))
        }
    }
}

pub struct OneOrMoreLookingBack<A, F>(A, F);

impl<A, F> OneOrMoreLookingBack<A, F> {
    pub fn new(seed_source: A, factory: F) -> Self {
        Self(seed_source, factory)
    }
}

impl<R, A, B, F> Parser<R> for OneOrMoreLookingBack<A, F>
where
    R: Reader,
    A: Parser<R>,
    B: Parser<R, Output = A::Output>,
    F: Fn(&A::Output) -> B,
{
    type Output = Vec<A::Output>;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, <R as Reader>::Err> {
        // get first item
        let mut r = reader;
        let (tmp, opt_first_item) = self.0.parse(r)?;
        r = tmp;
        match opt_first_item {
            Some(first_item) => {
                let mut result: Vec<A::Output> = vec![];
                loop {
                    let seed = if result.is_empty() {
                        &first_item
                    } else {
                        result.last().unwrap()
                    };
                    let parser = (self.1)(seed);
                    let (tmp, opt_item) = parser.parse(r)?;
                    r = tmp;
                    match opt_item {
                        Some(item) => {
                            result.push(item);
                        }
                        _ => {
                            break;
                        }
                    }
                }
                result.insert(0, first_item);
                Ok((r, Some(result)))
            }
            _ => Ok((r, None)),
        }
    }
}

/// Offers chaining methods that result in parsers the return multiple results.
pub trait ManyParser<R: Reader>: Parser<R> + Sized {
    /// Returns a parser that uses this parser to parse one or more items and
    /// collects them into a `Vec`.
    /// Parsing stops when the underlying parser returns `None`.
    fn one_or_more(self) -> OneOrMore<Self> {
        OneOrMore(self)
    }

    /// Returns a parser that uses this parser to parse zero or more items and
    /// collects them into a `Vec`.
    /// Parsing stops when the underlying parser returns `None`.
    fn zero_or_more(self) -> MapNoneToDefault<OneOrMore<Self>> {
        self.one_or_more().map_none_to_default()
    }

    /// Returns a parser that parses items separated by a delimiter.
    fn one_or_more_delimited_by<D, E>(
        self,
        delimiter: D,
        trailing_delimiter_err: E,
    ) -> OneOrMoreDelimited<Self, D, E> {
        OneOrMoreDelimited(self, delimiter, trailing_delimiter_err)
    }

    fn one_or_more_looking_back<S, F>(self, factory: F) -> OneOrMoreLookingBack<Self, F>
    where
        F: Fn(&Self::Output) -> S,
        S: Parser<R, Output = Self::Output>,
    {
        OneOrMoreLookingBack::new(self, factory)
    }
}

impl<R: Reader, T> ManyParser<R> for T where T: Parser<R> {}
