use rusty_common::*;
use rusty_pc::*;

use crate::ParseError;
use crate::core::comment::comment_as_string_p;
/// Separator between statements.
/// There are two cases, after a comment, or after a different kind of statement.
///
/// For the comment we have:
///
/// `EOL <ws | eol>*`
///
/// And for the rest of the cases we have:
///
/// ````text
/// <ws>* '\'' (undoing it)
/// <ws>* ':' <ws*>
/// <ws>* EOL <ws | eol>*
/// ```
use crate::input::RcStringView;
use crate::pc_specific::*;
use crate::tokens::{TokenMatcher, TokenType, any_token_of, peek_token, whitespace};

/// Parses a comment separator, which is the EOL,
/// followed optionally by any number of EOL or whitespace tokens.
pub fn comment_separator() -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    any_token_of!(TokenType::Eol).and(
        any_token_of!(TokenType::Eol, TokenType::Whitespace).many_allow_none(IgnoringManyCombiner),
        IgnoringBothCombiner,
    )
}

/// Common separator reads a separator between statements.
///
/// The steps are:
///
/// Skip whitespace.
/// If single quote, undo and return ok.
/// If found colon, store that information and continue reading.
/// If found EOL, store that information and continue reading.
/// If found anything else, stop.
/// If found colon after having found a colon or an EOL, undo it and stop.
/// So it's something like: one colon or multiple EOL, surrounded by optional whitespace.
///
/// The colon variant can be seen as:
/// ws* colon (ws | eol)*
///
/// The eol variant can be seen as:
/// ws* eol (ws | eol)*
///
/// The single quote variant can be seen as:
/// ws* ' (but undoing it and without reading anything after it)
///
/// Together it should be:
/// ws* (colon | eol) (ws | eol)*
/// ws* ' ! (where `!` stands for read and undo)
pub fn common_separator() -> impl Parser<RcStringView, Output = (), Error = ParseError> {
    opt_and(
        whitespace(),
        OrParser::new(vec![
            Box::new(any_token_of!(TokenType::Eol ; symbols = ':').and_opt(
                any_token_of!(TokenType::Eol, TokenType::Whitespace).zero_or_more(),
                |_, _| (),
            )),
            Box::new(no_separator_needed_before_comment()),
        ]),
        |_, _| (),
    )
}

pub fn no_separator_needed_before_comment()
-> impl Parser<RcStringView, Output = (), Error = ParseError> {
    // warning: cannot use filter_map because it will undo and we've already "undo" via "peek"
    peek_token().flat_map(|input, t| {
        if '\''.matches_token(&t) {
            Ok((input, ()))
        } else {
            default_parse_error(input)
        }
    })
}

pub fn peek_eof_or_statement_separator()
-> impl Parser<RcStringView, Output = (), Error = ParseError> {
    peek_token().flat_map_negate_none(|input, token| {
        if ':'.matches_token(&token)
            || '\''.matches_token(&token)
            || TokenType::Eol.matches_token(&token)
        {
            Ok((input, ()))
        } else {
            default_parse_error(input)
        }
    })
}

// TODO review all parsers that return a collection, implement some `accumulate` method
/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p()
-> impl Parser<RcStringView, Output = Vec<Positioned<String>>, Error = ParseError> {
    opt_and_keep_right(
        whitespace(),
        OptZip::new(comment_separator(), comment_as_string_p().with_pos())
            .one_or_more()
            .map(ZipValue::collect_right)
            .or_default(),
    )
}
