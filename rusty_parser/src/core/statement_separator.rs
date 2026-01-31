use rusty_common::*;
use rusty_pc::and::{IgnoringBothCombiner, opt_and};
use rusty_pc::many::IgnoringManyCombiner;
use rusty_pc::*;

use crate::ParserError;
use crate::core::comment::comment_as_string_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{TokenMatcher, TokenType, any_token_of, peek_token, whitespace_ignoring};

/// Parses a comment separator, which is the EOL,
/// followed optionally by any number of EOL or whitespace tokens.
pub fn comment_separator() -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Eol).and(eol_ws_zero_or_more(), IgnoringBothCombiner)
}

/// Parses any number of EOL or whitespace tokens.
fn eol_ws_zero_or_more() -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Eol, TokenType::Whitespace).many_allow_none(IgnoringManyCombiner)
}

/// Common separator reads a separator between statements.
///
/// ````text
/// <ws>* '\'' (undoing it)
/// <ws>* ':' <ws*>
/// <ws>* EOL <ws | eol>*
/// ```
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
pub fn common_separator() -> impl Parser<StringView, Output = (), Error = ParserError> {
    opt_and(
        whitespace_ignoring(),
        OrParser::new(vec![
            Box::new(eol_or_col_separator()),
            Box::new(no_separator_needed_before_comment()),
        ]),
        IgnoringBothCombiner,
    )
}

/// EOL or colon, followed by any number of EOL or whitespace tokens.
/// (eol | colon) (ws | eol)*
fn eol_or_col_separator() -> impl Parser<StringView, Output = (), Error = ParserError> {
    any_token_of!(TokenType::Eol ; symbols = ':').and(eol_ws_zero_or_more(), IgnoringBothCombiner)
}

pub fn no_separator_needed_before_comment()
-> impl Parser<StringView, Output = (), Error = ParserError> {
    // warning: cannot use filter_map because it will undo and we've already "undo" via "peek"
    peek_token().and_then(|t| {
        if '\''.matches_token(&t) {
            Ok(())
        } else {
            default_parse_error()
        }
    })
}

pub fn peek_eof_or_statement_separator() -> impl Parser<StringView, Output = (), Error = ParserError>
{
    peek_token().flat_map_negate_none(|token| {
        if ':'.matches_token(&token)
            || '\''.matches_token(&token)
            || TokenType::Eol.matches_token(&token)
        {
            Ok(())
        } else {
            default_parse_error()
        }
    })
}

/// Reads multiple comments and the surrounding whitespace.
/// Used for SELECT and TYPE statements to parse comments
/// that might be in-between keywords.
pub fn comments_in_between_keywords()
-> impl Parser<StringView, Output = Vec<Positioned<String>>, Error = ParserError> {
    eol_ws_zero_or_more().and_keep_right(comment_as_string_followed_by_separator().zero_or_more())
}

/// Parses a comment as a string, demanding that it is followed
/// by a separator (EOL), as this is supposed to be a
/// comment in-between keywords, so it can't be terminated by EOF.
fn comment_as_string_followed_by_separator()
-> impl Parser<StringView, Output = Positioned<String>, Error = ParserError> {
    comment_as_string_p()
        .with_pos()
        .and_keep_left(comment_separator().or_expected("EOL"))
}
