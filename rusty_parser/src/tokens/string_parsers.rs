use rusty_pc::Parser;
use rusty_pc::many::{IgnoringManyCombiner, StringManyCombiner};

use crate::ParserError;
use crate::input::StringView;
use crate::tokens::specific_str::{SpecificString};

/// Parses the specific string, case insensitive.
pub(super) fn specific_str(
    needle: String,
) -> impl Parser<StringView, Output = String, Error = ParserError> {
    SpecificString::new(needle, StringManyCombiner)
}

/// Parses the specific string, case insensitive, ignoring the output.
pub(super) fn specific_str_ignoring(
    needle: String,
) -> impl Parser<StringView, Output = (), Error = ParserError> {
    SpecificString::new(needle, IgnoringManyCombiner)
}
