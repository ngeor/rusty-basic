use crate::pc::*;
use crate::ParseError;

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
pub fn delimited_by<I: Tokenizer + 'static, P: Parser<I> + 'static, D: Parser<I> + 'static>(
    parser: P,
    delimiter: D,
    trailing_error: ParseError,
) -> impl Parser<I, Output = Vec<P::Output>> {
    parse_delimited_to_items(parser.and_opt_tuple(delimiter), trailing_error)
}

/// Gets a list of items separated by a delimiter.
/// The given parser already provides items and delimiters together.
/// Public because needed by built_ins to implement csv_allow_missing.
pub fn parse_delimited_to_items<I: Tokenizer + 'static, P, L>(
    parser: P,
    trailing_error: ParseError,
) -> impl Parser<I, Output = Vec<L>>
where
    P: Parser<I>,
    P::Output: Delimited<L>,
{
    parser
        .loop_while(Delimited::has_delimiter)
        .flat_map(move |items| map_items(items, trailing_error.clone()))
}

fn map_items<P, T>(items: Vec<P>, trailing_error: ParseError) -> ParseResult<Vec<T>, ParseError>
where
    P: Delimited<T>,
{
    if items
        .last()
        .map(Delimited::has_delimiter)
        .unwrap_or_default()
    {
        ParseResult::Err(trailing_error)
    } else {
        ParseResult::Ok(items.into_iter().map(Delimited::value).collect())
    }
}
