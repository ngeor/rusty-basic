use crate::common::*;
use crate::parser::built_ins;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim;
use crate::parser::do_loop;
use crate::parser::exit::statement_exit_p;
use crate::parser::for_loop;
use crate::parser::go_sub::{statement_go_sub_p, statement_return_p};
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::name::bare_name_p;
use crate::parser::on_error::statement_on_error_go_to_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::{keyword_followed_by_whitespace_p, keyword_p, PcSpecific};
use crate::parser::resume::statement_resume_p;
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;

pub fn statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    statement_label_p()
        .or(single_line_statement_p())
        .or(if_block::if_block_p())
        .or(for_loop::for_loop_p())
        .or(select_case::select_case_p())
        .or(while_wend::while_wend_p())
        .or(do_loop::do_loop_p())
        .or(illegal_starting_keywords())
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// excluding comments.
pub fn single_line_non_comment_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    dim::dim_p()
        .or(constant::constant_p())
        .or(built_ins::parse_built_in_p())
        .or(sub_call::sub_call_or_assignment_p())
        .or(statement_go_to_p())
        .or(statement_go_sub_p())
        .or(statement_return_p())
        .or(statement_exit_p())
        .or(statement_on_error_go_to_p())
        .or(statement_resume_p())
        .or(end::parse_end_p())
        .or(system::parse_system_p())
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    comment::comment_p().or(single_line_non_comment_statement_p())
}

fn statement_label_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    name::bare_name_p()
        .and(item_p(':'))
        .keep_left()
        .map(|l| Statement::Label(l))
}

fn statement_go_to_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_followed_by_whitespace_p(Keyword::GoTo)
        .and_demand(bare_name_p().or_syntax_error("Expected: label"))
        .map(|(_, l)| Statement::GoTo(l))
}

fn illegal_starting_keywords<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Wend)
        .or(keyword_p(Keyword::Else))
        .and_then(|(k, _)| match k {
            Keyword::Wend => Err(QError::WendWithoutWhile),
            Keyword::Else => Err(QError::ElseWithoutIf),
            Keyword::Loop => Err(QError::syntax_error("LOOP without DO")),
            _ => panic!("Parser should not have parsed {}", k),
        })
}

mod end {
    use super::*;
    use crate::parser::pc_specific::keyword_choice_p;
    use crate::parser::statement_separator::EofOrStatementSeparator;

    pub fn parse_end_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::End)
            .and(
                opt_whitespace_p(false)
                    .and(AfterEndSeparator {})
                    .map(|(l, r)| {
                        let mut s: String = String::new();
                        s.push_str(&l);
                        s.push_str(&r);
                        s
                    })
                    .peek(),
            )
            .map(|_| Statement::End)
    }

    /// Parses the next token after END. If it is one of the valid keywords that
    /// can follow END, it is undone so that the entire parsing will be undone.
    /// Otherwise, it demands that we find an end-of-statement terminator.
    struct AfterEndSeparator {}

    impl<R> Parser<R> for AfterEndSeparator
    where
        R: Reader<Item = char, Err = QError>,
    {
        type Output = String;

        fn parse(&mut self, reader: R) -> ReaderResult<R, Self::Output, R::Err> {
            let (reader, opt_result) = allowed_keywords_after_end().parse(reader)?;
            match opt_result {
                Some((_, s)) => {
                    // undo and return None, as another parser will handle this
                    Ok((reader.undo(s), None))
                }
                _ => {
                    let (reader, opt_str) = EofOrStatementSeparator::new().parse(reader)?;
                    match opt_str {
                        Some(s) => Ok((reader, Some(s))),
                        _ => {
                            // error
                            Err((reader, QError::syntax_error("Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement")))
                        }
                    }
                }
            }
        }
    }

    fn allowed_keywords_after_end<R>() -> impl Parser<R, Output = (Keyword, String)>
    where
        R: Reader<Item = char>,
    {
        keyword_choice_p(&[
            Keyword::Function,
            Keyword::If,
            Keyword::Select,
            Keyword::Sub,
            Keyword::Type,
        ])
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_parser_err;

        #[test]
        fn test_sub_call_end_no_args_allowed() {
            assert_parser_err!(
                "END 42",
                QError::syntax_error(
                    "Expected: DEF or FUNCTION or IF or SELECT or SUB or TYPE or end-of-statement"
                )
            );
        }
    }
}

mod system {
    use super::*;
    use crate::parser::statement_separator::EofOrStatementSeparator;

    pub fn parse_system_p<R>() -> impl Parser<R, Output = Statement>
    where
        R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    {
        keyword_p(Keyword::System)
            .and_demand(
                opt_whitespace_p(false)
                    .and(EofOrStatementSeparator::new())
                    .map(|(l, r)| {
                        let mut s: String = String::new();
                        s.push_str(&l);
                        s.push_str(&r);
                        s
                    })
                    .peek()
                    .or_syntax_error("Expected: end-of-statement"),
            )
            .map(|_| Statement::System)
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_parser_err;
        use crate::common::*;

        #[test]
        fn test_sub_call_system_no_args_allowed() {
            assert_parser_err!(
                "SYSTEM 42",
                QError::syntax_error("Expected: end-of-statement"),
                1,
                7
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::{PrintNode, Statement, TopLevelToken};

    use super::super::test_utils::*;

    #[test]
    fn test_top_level_comment() {
        let input = "' closes the file";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Comment(" closes the file".to_string(),))
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
                TopLevelToken::Statement(Statement::Print(PrintNode::one(42.as_lit_expr(1, 9))))
                    .at_rc(1, 3)
            ]
        );
    }
}
