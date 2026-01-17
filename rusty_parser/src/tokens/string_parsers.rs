use rusty_pc::text::any_char;
use rusty_pc::{IgnoringManyCombiner, Many, ManyCombiner, Map, Parser, StringManyCombiner};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::specific_str::{SpecificStr, SpecificString};

/// Parses one or more characters that match the given predicate
/// and returns a String.
pub(super) fn many<F>(
    predicate: F,
) -> impl Parser<RcStringView, Output = String, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    many_collecting(predicate, StringManyCombiner)
}

/// Parses one or more characters that match the given predicate,
/// but ignores them, returning just `()`.
#[allow(dead_code)]
pub(super) fn many_ignoring<F>(
    predicate: F,
) -> impl Parser<RcStringView, Output = (), Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    many_collecting(predicate, IgnoringManyCombiner)
}

/// Parses one or more characters that match the given predicate,
/// collecting them with the given combiner.
pub(super) fn many_collecting<F, C, O>(
    predicate: F,
    combiner: C,
) -> impl Parser<RcStringView, Output = O, Error = ParseError>
where
    F: Fn(&char) -> bool,
    C: ManyCombiner<char, O>,
    O: Default,
{
    any_char().filter(predicate).many(combiner)
}

pub(super) fn one(ch: char) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    any_char()
        .filter(move |c: &char| *c == ch)
        .map(String::from)
}

/// Parses the specific string, case insensitive.
pub(super) fn specific(
    needle: &str,
) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    SpecificStr::new(needle, StringManyCombiner)
}

/// Parses the specific string, case insensitive.
pub(super) fn specific_owned(
    needle: String,
) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    SpecificString::new(needle, StringManyCombiner)
}

/// Parses the specific string, case insensitive, ignoring the output.
pub(super) fn specific_ignoring(
    needle: String,
) -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    SpecificString::new(needle, IgnoringManyCombiner)
}
