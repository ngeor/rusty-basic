use rusty_pc::{Filter, Many, Map, Parser, StringManyCombiner};

use crate::ParseError;
use crate::input::RcStringView;
use crate::tokens::any_char::AnyChar;
use crate::tokens::specific_str::{SpecificStr, SpecificString};

pub(super) fn many<F>(
    predicate: F,
) -> impl Parser<RcStringView, Output = String, Error = ParseError>
where
    F: Fn(&char) -> bool,
{
    AnyChar.filter(predicate).many(StringManyCombiner)
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
