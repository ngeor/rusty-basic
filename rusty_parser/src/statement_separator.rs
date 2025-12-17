/// Separator between statements.
/// There are two cases, after a comment, or after a different kind of statement.
///
/// For the comment we have:
///
/// `<ws>* EOL <ws | eol>*`
///
/// And for the rest of the cases we have:
///
/// ````text
/// <ws>* '\'' (undoing it)
/// <ws>* ':' <ws*>
/// <ws>* EOL <ws | eol>*
/// ```
use crate::comment::comment_as_string_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::ParseError;
use rusty_common::*;

pub fn comment_separator<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    OptAndPC::new(whitespace(), any_token_of(TokenType::Eol))
        .and_opt(any_token_of_two(TokenType::Eol, TokenType::Whitespace))
        .map(|_| ())
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
pub fn common_separator<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    OptAndPC::new(
        whitespace(),
        OrParser::new(vec![
            Box::new(
                any_token_of_two(TokenType::Colon, TokenType::Eol)
                    .and_opt(any_token_of_two(TokenType::Eol, TokenType::Whitespace).zero_or_more())
                    .map(|_| ()),
            ),
            Box::new(no_separator_needed_before_comment()),
        ]),
    )
    .map(|_| ())
}

pub fn no_separator_needed_before_comment<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    // warning: cannot use filter_map because it will undo and we've already "undo" via "peek"
    peek_token().and_then(|t| {
        if TokenType::SingleQuote.matches(&t) {
            ParseResult::Ok(())
        } else {
            ParseResult::None
        }
    })
}

pub fn peek_eof_or_statement_separator<I: Tokenizer + 'static>() -> impl Parser<I, Output = ()> {
    peek_token().and_then_ok_err(
        |token| {
            if TokenType::Colon.matches(&token)
                || TokenType::SingleQuote.matches(&token)
                || TokenType::Eol.matches(&token)
            {
                ParseResult::Ok(())
            } else {
                ParseResult::None
            }
        },
        // allow EOF
        || ParseResult::Ok(()),
    )
}

// TODO review all parsers that return a collection, implement some `accumulate` method
/// Reads multiple comments and the surrounding whitespace.
pub fn comments_and_whitespace_p<I: Tokenizer + 'static>(
) -> impl Parser<I, Output = Vec<Positioned<String>>> {
    OptAndPC::new(
        whitespace(),
        OptZip::new(comment_separator(), comment_as_string_p().with_pos())
            .one_or_more()
            .map(ZipValue::collect_right)
            .allow_default(),
    )
    .keep_right()
}
