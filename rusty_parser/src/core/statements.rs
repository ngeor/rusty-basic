use rusty_common::Positioned;
use rusty_pc::*;

use crate::core::statement::statement_p;
use crate::core::statement_separator::{comment_separator, common_separator};
use crate::input::StringView;
use crate::pc_specific::*;
use crate::*;

macro_rules! zero_or_more_statements {
    ($exit:expr, ParserError::$err:ident) => {
        $crate::core::statements::zero_or_more_statements_p([$exit], Some(ParserError::$err))
    };
    ($($exit:expr),+) => {
        $crate::core::statements::zero_or_more_statements_p( [$($exit),+], None)
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
    exit_keywords: impl IntoIterator<Item = Keyword> + 'static,
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
    exit_keywords: impl IntoIterator<Item = Keyword> + 'static,
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, bool, Output = StatementPos, Error = ParserError> + SetContext<bool> {
    one_statement_or_exit_keyword_p(exit_keywords, custom_err).flat_map(
        |statement_or_exit_keyword| match statement_or_exit_keyword {
            // we parsed a statement, return it
            StatementOrExitKeyword::Statement(s) => Ok(s),
            // we detected an exit keyword, stop parsing
            StatementOrExitKeyword::ExitKeyword(_keyword) => default_parse_error(),
        },
    )
}

/// Parses either one statement or the exit keyword (peeking).
/// Either one must be preceded by the appropriate separator,
/// which depending on the context (previously parsed statement)
/// is either the comment separator (EOL) or the regular separator (EOL or colon).
fn one_statement_or_exit_keyword_p(
    exit_keywords: impl IntoIterator<Item = Keyword> + 'static,
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
    ctx_parser()
        .map(|last_statement_was_comment| {
            if last_statement_was_comment {
                comment_separator().boxed().no_context()
            } else {
                common_separator().boxed().no_context()
            }
        })
        .flatten()
        .or_expected("end-of-statement")
}

fn find_exit_keyword_or_demand_statement_p(
    exit_keywords: impl IntoIterator<Item = Keyword> + 'static,
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    find_exit_keyword_p(exit_keywords, custom_err).or(demand_statement_p())
}

fn find_exit_keyword_p(
    exit_keywords: impl IntoIterator<Item = Keyword> + 'static,
    custom_err: Option<ParserError>,
) -> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    // the first parser will return:
    // Ok if it finds the keyword (peeking)
    // Err(false) if it finds something else
    // Err(true) if it finds EOF
    let p = keyword_p(exit_keywords, true)
        .peek()
        // Ok(None) if it finds the keyword
        .map(StatementOrExitKeyword::ExitKeyword);

    match custom_err {
        Some(err) => p.with_fatal_err(err).boxed(),
        None => p.boxed(),
    }
}

fn demand_statement_p()
-> impl Parser<StringView, Output = StatementOrExitKeyword, Error = ParserError> {
    // needs to be lazy otherwise stackoverflow
    lazy(statement_p)
        .with_pos()
        .map(StatementOrExitKeyword::Statement)
        .or_expected("statement")
}

enum StatementOrExitKeyword {
    Statement(StatementPos),
    ExitKeyword(Keyword),
}
