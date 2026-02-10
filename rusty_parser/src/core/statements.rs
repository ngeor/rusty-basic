use rusty_common::Positioned;
use rusty_pc::and::KeepRightCombiner;
use rusty_pc::*;

use crate::core::statement::statement_p;
use crate::core::statement_separator::{comment_separator, common_separator};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::{TokenMatcher, peek_token};
use crate::*;

macro_rules! zero_or_more_statements {
    ($exit:expr, ParserError::$err:ident) => {
        $crate::core::statements::zero_or_more_statements_p(&[$exit], Some(ParserError::$err))
    };
    ($($exit:expr),+) => {
        $crate::core::statements::zero_or_more_statements_p( &[$($exit),+], None)
    };
}

pub(crate) use zero_or_more_statements;

/// Parses zero or more statements after the beginning of a statement that expects a block,
/// e.g. DO ... LOOP, SUB ... END SUB, etc.
///
/// The given keywords determine when parsing should stop (without parsing the keywords themselves).
/// e.g. in case of a FOR, stop parsing when NEXT is detected.
///
/// The custom error can be used to specify a custom error when the exit keyword is not found.
pub fn zero_or_more_statements_p(
    exit_keywords: &[Keyword],
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, Output = Statements, Error = ParserError> {
    one_statement_p(exit_keywords, custom_err)
        .zero_or_more()
        // Initialize the context before the loop of "zero_or_more" starts.
        // The context indicates if the previous statement was a comment.
        .init_context(false)
}

/// Either parses one statement or detects the exit keyword and stops parsing.
fn one_statement_p(
    exit_keywords: &[Keyword],
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, bool, Output = StatementPos, Error = ParserError> + SetContext<bool> {
    one_statement_or_exit_keyword_p(exit_keywords, custom_err).and_then(
        |statement_or_exit_keyword| match statement_or_exit_keyword {
            // we parsed a statement, return it
            StatementOrExitKeyword::Statement(s) => Ok(s),
            // we detected an exit keyword, stop parsing
            StatementOrExitKeyword::ExitKeyword => default_parse_error(),
        },
    )
}

/// Parses either one statement or the exit keyword (peeking).
/// Either one must be preceded by the appropriate separator,
/// which depending on the context (previously parsed statement)
/// is either the comment separator (EOL) or the regular separator (EOL or colon).
fn one_statement_or_exit_keyword_p(
    exit_keywords: &[Keyword],
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, bool, Output = StatementOrExitKeyword, Error = ParserError> + SetContext<bool>
{
    ThenWithLeftParser::new(
        // must parse the separator
        ctx_demand_separator_p(),
        // must parse the statement or peek the exit keyword
        find_exit_keyword_or_demand_statement_p(exit_keywords, custom_err),
        // populate the context of the separator for the next iteration
        is_comment,
        // keep only the statement or the peeked exit keyword
        KeepRightCombiner,
    )
}

fn is_comment(statement_or_exit_keyword: &StatementOrExitKeyword) -> bool {
    matches!(
        statement_or_exit_keyword,
        StatementOrExitKeyword::Statement(Positioned {
            element: Statement::Comment(_),
            ..
        })
    )
}

/// A statement separator that is aware if the previously parsed statement
/// was a comment or not.
fn ctx_demand_separator_p()
-> impl Parser<StringView, bool, Output = (), Error = ParserError> + SetContext<bool> {
    // TODO consolidate the two separate separator functions, they are almost never used elsewhere
    IifParser::new(
        // last statement was comment
        comment_separator(),
        // last statement was not comment
        common_separator(),
    )
    .or_expected("end-of-statement")
}

fn find_exit_keyword_or_demand_statement_p(
    exit_keywords: &[Keyword],
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    find_exit_keyword_p(exit_keywords, custom_err).or(demand_statement_p())
}

fn find_exit_keyword_p(
    exit_keywords: &[Keyword],
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    // the parser will return:
    // Ok if it finds the keyword (peeking)
    // Soft error if it finds something else
    // Fatal error if it finds EOF
    peek_token().to_option().and_then(move |opt_token| {
        match opt_token {
            Some(token) => {
                for exit_keyword in exit_keywords {
                    if exit_keyword.matches_token(&token) {
                        return Ok(StatementOrExitKeyword::ExitKeyword);
                    }
                }

                Err(ParserError::expected(&to_syntax_err(exit_keywords.iter())))
            }
            None => {
                // eof is fatal
                match &custom_err {
                    Some(err) => Err(err.clone()),
                    None => {
                        let s = to_syntax_err(exit_keywords.iter());
                        Err(ParserError::expected(&s).to_fatal())
                    }
                }
            }
        }
    })
}

fn demand_statement_p()
-> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    // needs to be lazy otherwise stackoverflow
    lazy(statement_p)
        .with_pos()
        .map(StatementOrExitKeyword::Statement)
        .or_expected("statement")
}

#[allow(clippy::large_enum_variant)]
enum StatementOrExitKeyword {
    Statement(StatementPos),
    ExitKeyword,
}
