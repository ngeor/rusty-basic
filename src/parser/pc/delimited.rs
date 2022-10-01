use crate::common::QError;
use crate::parser::pc::{Parser, ParserBase, ZipValue};

/// Represents a value that has is followed by optional delimiter.
pub trait Delimited<T> {
    fn has_delimiter(&self) -> bool;
    fn value(self) -> T;
}

// used by opt_zip

impl<L, R> Delimited<Option<L>> for ZipValue<L, R> {
    fn has_delimiter(&self) -> bool {
        self.has_right()
    }

    fn value(self) -> Option<L> {
        self.left()
    }
}

// used by and_opt

impl<L, R> Delimited<L> for (L, Option<R>) {
    fn has_delimiter(&self) -> bool {
        let (_, right) = self;
        right.is_some()
    }

    fn value(self) -> L {
        let (left, _) = self;
        left
    }
}

/// Gets a list of items separated by a delimiter.
/// One or more if parser, zero or more if non-opt-parser.
pub fn delimited_by<P: Parser, D: Parser>(
    parser: P,
    delimiter: D,
    allow_empty: bool,
    trailing_error: QError,
) -> impl Parser<Output = Vec<P::Output>> {
    parse_delimited_to_items(parser.and_opt(delimiter), allow_empty, trailing_error)
}

/// Gets a list of items separated by a delimiter.
/// The given parser already provides items and delimiters together.
/// Public because needed by built_ins to implement csv_allow_missing.
pub fn parse_delimited_to_items<P, L>(
    parser: P,
    allow_empty: bool,
    trailing_error: QError,
) -> impl Parser<Output = Vec<L>>
where
    P: Parser,
    P::Output: Delimited<L>,
{
    parser
        .loop_while(Delimited::has_delimiter, allow_empty)
        .and_then(move |items| map_items(items, trailing_error.clone()))
}

// non opt

pub fn delimited_by_non_opt<P: Parser, D: Parser>(
    parser: P,
    delimiter: D,
    trailing_error: QError,
) -> impl Parser<Output = Vec<P::Output>> {
    zero_or_more_delimited_non_opt(parser.and_opt(delimiter), trailing_error)
}

fn zero_or_more_delimited_non_opt<P, L>(
    parser: P,
    trailing_error: QError,
) -> impl Parser<Output = Vec<L>>
where
    P: Parser,
    P::Output: Delimited<L>,
{
    parser
        .loop_while(Delimited::has_delimiter, true)
        .and_then(move |items| map_items(items, trailing_error.clone()))
}

fn map_items<P, T>(items: Vec<P>, trailing_error: QError) -> Result<Vec<T>, QError>
where
    P: Delimited<T>,
{
    if items
        .last()
        .map(Delimited::has_delimiter)
        .unwrap_or_default()
    {
        Err(trailing_error)
    } else {
        Ok(items.into_iter().map(Delimited::value).collect())
    }
}
