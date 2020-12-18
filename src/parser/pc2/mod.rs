pub mod binary;
pub mod sources;
pub mod text;
pub mod unary;
pub mod unary_fn;

use crate::parser::pc::{Reader, ReaderResult, Undo};
use crate::parser::pc2::binary::{BinaryParser, LeftAndOptRight, OptLeftAndRight};
use crate::parser::pc2::text::{read_one_or_more_whitespace_p, Whitespace};
use crate::parser::pc2::unary_fn::UnaryFnParser;

pub trait Parser<R>: Sized
where
    R: Reader,
{
    type Output;

    fn parse(&self, reader: R) -> ReaderResult<R, Self::Output, R::Err>;

    fn convert_to_fn(self) -> Box<dyn Fn(R) -> ReaderResult<R, Self::Output, R::Err>>
    where
        Self: Sized + 'static,
    {
        let x = self;
        Box::new(move |reader| x.parse(reader))
    }

    //
    // collection parsers
    //

    fn one_or_more(self) -> OneOrMore<Self> {
        OneOrMore(self)
    }

    fn one_or_more_delimited_by<D, F>(
        self,
        delimiter: D,
        trailing_delimiter_err_fn: F,
    ) -> OneOrMoreDelimited<Self, D, F> {
        OneOrMoreDelimited(self, delimiter, trailing_delimiter_err_fn)
    }

    //
    // text parsers
    //

    fn surrounded_by_opt_ws(
        self,
    ) -> OptLeftAndRight<text::Whitespace<R>, LeftAndOptRight<Self, text::Whitespace<R>>>
    where
        R: Reader<Item = char>,
    {
        OptLeftAndRight::new(Whitespace::new(), self.followed_by_opt_ws())
    }

    fn followed_by_opt_ws(self) -> binary::LeftAndOptRight<Self, text::Whitespace<R>>
    where
        R: Reader<Item = char>,
    {
        self.and_opt(read_one_or_more_whitespace_p())
    }

    fn stringify(self) -> text::Stringify<Self> {
        text::Stringify::new(self)
    }
}

//
// sources
//

/// A parser that reads the next item from the reader.
pub fn any_p<R: Reader>() -> sources::Any<R> {
    sources::Any::<R>::new()
}

/// A parser that reads the next item from the reader if it meets the given predicate.
pub fn read_one_if_p<R, F>(predicate: F) -> impl Parser<R, Output = R::Item>
where
    R: Reader,
    R::Item: Copy,
    F: Fn(R::Item) -> bool,
{
    any_p::<R>().filter_reader_item(predicate)
}

/// A parser that reads the next item if it matches the given parameter.
pub fn read_one_p<R>(item: R::Item) -> impl Parser<R, Output = R::Item>
where
    R: Reader,
    R::Item: Copy + Eq + 'static,
{
    read_one_if_p(move |x| x == item)
}

//
// OneOrMore
//

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

// One or more with delimiter

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
