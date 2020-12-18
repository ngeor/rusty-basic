/// This module has parsers that return one or more items.
use super::{Parser, Reader, ReaderResult};

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
/// - F: A function that returns an error in case of trailing delimiters.
pub struct OneOrMoreDelimited<S, D, F>(S, D, F);

impl<R, S, D, F> Parser<R> for OneOrMoreDelimited<S, D, F>
where
    R: Reader,
    S: Parser<R>,
    D: Parser<R>,
    F: Fn() -> R::Err,
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
                        return Err((reader, (self.2)()));
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

/// Offers chaining methods that result in parsers the return multiple results.
pub trait ManyParser<R: Reader>: Parser<R> {
    /// Returns a parser that uses this parser to parse items and collects them
    /// into a `Vec`. Parsing stops when the underlying parser returns `None`.
    fn one_or_more(self) -> OneOrMore<Self> {
        OneOrMore(self)
    }

    /// Returns a parser that parses items separated by a delimiter.
    fn one_or_more_delimited_by<D, F>(
        self,
        delimiter: D,
        trailing_delimiter_err_fn: F,
    ) -> OneOrMoreDelimited<Self, D, F> {
        OneOrMoreDelimited(self, delimiter, trailing_delimiter_err_fn)
    }
}

impl<R: Reader, T> ManyParser<R> for T where T: Parser<R> {}
