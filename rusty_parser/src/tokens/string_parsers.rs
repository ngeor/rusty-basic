use rusty_pc::{Filter, IgnoringManyCombiner, Many, Map, Parser, StringManyCombiner};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::any_char::AnyChar;
use crate::tokens::specific_str::{SpecificStr, SpecificString};

/// Parses one or more characters that match the given predicate
/// and returns a String.
pub(super) fn many<F>(
    predicate: F,
) -> impl Parser<RcStringView, Output = String, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    AnyChar.filter(predicate).many(StringManyCombiner)
}

/// Parses one or more characters that match the given predicate,
/// but ignores them, returning just `()`.
pub(super) fn many_ignoring<F>(
    predicate: F,
) -> impl Parser<RcStringView, Output = (), Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    AnyChar.filter(predicate).many(IgnoringManyCombiner)
}

pub(super) fn one(ch: char) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    AnyChar.filter(move |c| *c == ch).map(String::from)
}

pub(super) fn specific(
    needle: &str,
) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    SpecificStr::new(needle)
}

pub(super) fn specific_owned(
    needle: String,
) -> impl Parser<RcStringView, Output = String, Error = ParseError> {
    SpecificString::new(needle)
}
