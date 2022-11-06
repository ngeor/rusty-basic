use crate::constant;
use crate::dim;
use crate::do_loop;
use crate::exit::statement_exit_p;
use crate::for_loop;
use crate::go_sub::{statement_go_sub_p, statement_return_p};
use crate::if_block;
use crate::lazy_parser;
use crate::name::{bare_name_with_dots, identifier_with_dots};
use crate::on_error::statement_on_error_go_to_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::print;
use crate::resume::statement_resume_p;
use crate::select_case;
use crate::sub_call;
use crate::types::*;
use crate::while_wend;
use crate::{comment, ParseError};

lazy_parser!(pub fn statement_p<Output = Statement> ; struct LazyStatementP ; Alt8::new(
    statement_label_p(),
    single_line_statement_p(),
    if_block::if_block_p(),
    for_loop::for_loop_p(),
    select_case::select_case_p(),
    while_wend::while_wend_p(),
    do_loop::do_loop_p(),
    illegal_starting_keywords(),
));

// Tries to read a statement that is allowed to be on a single line IF statement,
// excluding comments.
lazy_parser!(pub fn single_line_non_comment_statement_p<Output = Statement> ; struct SingleLineNonCommentStatement ; Alt15::new(
    dim::dim_p(),
    dim::redim_p(),
    constant::constant_p(),
    super::built_ins::parse(),
    print::parse_print_p(),
    print::parse_lprint_p(),
    sub_call::sub_call_or_assignment_p(),
    statement_go_to_p(),
    statement_go_sub_p(),
    statement_return_p(),
    statement_exit_p(),
    statement_on_error_go_to_p(),
    statement_resume_p(),
    end::parse_end_p(),
    system::parse_system_p()));

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p() -> impl Parser<Output = Statement> {
    comment::comment_p().or(single_line_non_comment_statement_p())
}

fn statement_label_p() -> impl Parser<Output = Statement> {
    // labels can have dots
    identifier_with_dots()
        .and(colon())
        .keep_left()
        .map(|tokens| Statement::Label(token_list_to_bare_name(tokens)))
}

fn statement_go_to_p() -> impl Parser<Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .then_demand(bare_name_with_dots().or_syntax_error("Expected: label"))
        .map(Statement::GoTo)
}

/// A parser that fails if an illegal starting keyword is found.
fn illegal_starting_keywords() -> impl Parser<Output = Statement> + 'static {
    keyword_map(&[
        (Keyword::Wend, ParseError::WendWithoutWhile),
        (Keyword::Else, ParseError::ElseWithoutIf),
        (Keyword::Loop, ParseError::LoopWithoutDo),
        (Keyword::Next, ParseError::NextWithoutFor),
    ])
    .and_then(Err)
}

mod end {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::statement_separator::peek_eof_or_statement_separator;
    use crate::{Keyword, Statement};

    pub fn parse_end_p() -> impl Parser<Output = Statement> {
        keyword(Keyword::End)
            .then_demand(after_end_separator())
            .map(|_| Statement::End)
    }

    /// Parses the next token after END. If it is one of the valid keywords that
    /// can follow END, it is undone so that the entire parsing will be undone.
    /// Otherwise, it demands that we find an end-of-statement terminator.
    fn after_end_separator() -> impl Parser<Output = ()> + NonOptParser {
        Alt2::new(
            whitespace_and_allowed_keyword_after_end(),
            opt_ws_and_eof_or_statement_separator(),
        )
        .peek()
        .or_syntax_error(
            "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement",
        )
    }

    // Vec to be able to undo
    fn whitespace_and_allowed_keyword_after_end() -> impl Parser<Output = TokenList> {
        whitespace()
            .and(allowed_keywords_after_end())
            .map(|(l, (_, r))| vec![l, r])
    }

    fn allowed_keywords_after_end() -> impl Parser<Output = (Keyword, Token)> {
        keyword_choice(&[
            Keyword::Function,
            Keyword::If,
            Keyword::Select,
            Keyword::Sub,
            Keyword::Type,
        ])
    }

    fn opt_ws_and_eof_or_statement_separator() -> impl Parser<Output = TokenList> {
        whitespace()
            .allow_none()
            .and(peek_eof_or_statement_separator())
            .map(|(opt_ws, _)| opt_ws.into_iter().collect())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::ParseError;

        #[test]
        fn test_sub_call_end_no_args_allowed() {
            assert_parser_err!(
                "END 42",
                ParseError::syntax_error(
                    "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement"
                )
            );
        }
    }
}

mod system {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::statement_separator::peek_eof_or_statement_separator;
    use crate::{Keyword, Statement};

    pub fn parse_system_p() -> impl Parser<Output = Statement> {
        keyword(Keyword::System)
            .then_demand(
                OptAndPC::new(whitespace(), peek_eof_or_statement_separator())
                    .or_syntax_error("Expected: end-of-statement"),
            )
            .map(|_| Statement::System)
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::ParseError;

        #[test]
        fn test_sub_call_system_no_args_allowed() {
            assert_parser_err!(
                "SYSTEM 42",
                ParseError::syntax_error("Expected: end-of-statement"),
                1,
                7
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

    #[test]
    fn test_global_comment() {
        let input = "' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Comment(" closes the file".to_string(),))
                    .at_rc(1, 1)
            ]
        );
    }

    #[test]
    fn colon_separator_at_start_of_line() {
        let input = ": PRINT 42";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::Statement(Statement::Print(Print::one(42.as_lit_expr(1, 9))))
                    .at_rc(1, 3)
            ]
        );
    }
}
