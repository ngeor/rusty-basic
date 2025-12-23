use crate::comment;
use crate::constant;
use crate::dim;
use crate::do_loop;
use crate::error::ParseError;
use crate::exit::statement_exit_p;
use crate::for_loop;
use crate::go_sub::{statement_go_sub_p, statement_return_p};
use crate::if_block;
use crate::lazy_parser;
use crate::name::token_list_to_bare_name;
use crate::name::{bare_name_with_dots, identifier_with_dots};
use crate::on_error::statement_on_error_go_to_p;
use crate::pc::*;
use crate::pc_specific::*;
use crate::print;
use crate::resume::statement_resume_p;
use crate::select_case;
use crate::specific::*;
use crate::sub_call;
use crate::while_wend;

lazy_parser!(pub fn statement_p<I = RcStringView, Output = Statement> ; struct LazyStatementP ; OrParser::new(vec![
    Box::new(statement_label_p()),
    Box::new(single_line_statement_p()),
    Box::new(if_block::if_block_p()),
    Box::new(for_loop::for_loop_p()),
    Box::new(select_case::select_case_p()),
    Box::new(while_wend::while_wend_p()),
    Box::new(do_loop::do_loop_p()),
    Box::new(illegal_starting_keywords()),
]));

// Tries to read a statement that is allowed to be on a single line IF statement,
// excluding comments.
lazy_parser!(pub fn single_line_non_comment_statement_p<I = RcStringView, Output = Statement> ; struct SingleLineNonCommentStatement ; OrParser::new(vec![
    Box::new(dim::dim_p()),
    Box::new(dim::redim_p()),
    Box::new(constant::constant_p()),
    Box::new(super::built_ins::parse()),
    Box::new(print::parse_print_p()),
    Box::new(print::parse_lprint_p()),
    Box::new(sub_call::sub_call_or_assignment_p()),
    Box::new(statement_go_to_p()),
    Box::new(statement_go_sub_p()),
    Box::new(statement_return_p()),
    Box::new(statement_exit_p()),
    Box::new(statement_on_error_go_to_p()),
    Box::new(statement_resume_p()),
    Box::new(end::parse_end_p()),
    Box::new(system::parse_system_p())
]));

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p() -> impl Parser<RcStringView, Output = Statement> {
    comment::comment_p().or(single_line_non_comment_statement_p())
}

fn statement_label_p() -> impl Parser<RcStringView, Output = Statement> {
    // labels can have dots
    identifier_with_dots().and(colon(), |tokens, _| {
        Statement::Label(token_list_to_bare_name(tokens))
    })
}

fn statement_go_to_p() -> impl Parser<RcStringView, Output = Statement> {
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_without_undo_keep_right(bare_name_with_dots().or_syntax_error("Expected: label"))
        .map(Statement::GoTo)
}

/// A parser that fails if an illegal starting keyword is found.
fn illegal_starting_keywords() -> impl Parser<RcStringView, Output = Statement> {
    keyword_map(&[
        (Keyword::Wend, ParseError::WendWithoutWhile),
        (Keyword::Else, ParseError::ElseWithoutIf),
        (Keyword::Loop, ParseError::LoopWithoutDo),
        (Keyword::Next, ParseError::NextWithoutFor),
    ])
    .flat_map(|input, err| Err((true, input, err)))
}

mod end {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::specific::{Keyword, Statement};

    pub fn parse_end_p() -> impl Parser<RcStringView, Output = Statement> {
        keyword(Keyword::End).map(|_| Statement::End)
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::error::ParseError;

        #[test]
        fn test_sub_call_end_no_args_allowed() {
            assert_parser_err!(
                "END 42",
                ParseError::syntax_error(
                    // TODO FIXME this was originally like this:
                    // "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement"
                    "No separator: 42"
                )
            );
        }
    }
}

mod system {
    use crate::pc::*;
    use crate::pc_specific::*;
    use crate::specific::{Keyword, Statement};
    use crate::statement_separator::peek_eof_or_statement_separator;

    pub fn parse_system_p() -> impl Parser<RcStringView, Output = Statement> {
        keyword(Keyword::System).and_without_undo(
            opt_and_tuple(whitespace(), peek_eof_or_statement_separator())
                .or_syntax_error("Expected: end-of-statement"),
            |_, _| Statement::System,
        )
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::error::ParseError;

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
    use crate::specific::*;
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
