use crate::common::*;
use crate::parser::built_ins;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim;
use crate::parser::for_loop;
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::*;
use crate::parser::pc2::{LazyFnParser, Parser};
use crate::parser::pc_specific::*;
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;

pub fn statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    LazyFnParser::new(statement)
}

#[deprecated]
pub fn statement<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    or_vec(vec![
        dim::dim_p().box_dyn().convert_to_fn(),
        constant::constant_p().box_dyn().convert_to_fn(),
        comment::comment(),
        built_ins::parse_built_in_p().box_dyn().convert_to_fn(),
        statement_label(),
        sub_call::sub_call_or_assignment(),
        if_block::if_block_p().box_dyn().convert_to_fn(),
        for_loop::for_loop(),
        select_case::select_case_p().box_dyn().convert_to_fn(),
        while_wend::while_wend_p().box_dyn().convert_to_fn(),
        statement_go_to(),
        statement_on_error_go_to(),
        statement_illegal_keywords(),
    ])
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// excluding comments.
#[deprecated]
pub fn single_line_non_comment_statement<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    or_vec(vec![
        dim::dim(),
        constant::constant_p().convert_to_fn(),
        built_ins::parse_built_in(),
        sub_call::sub_call_or_assignment(),
        statement_go_to(),
        statement_on_error_go_to(),
    ])
}

pub fn single_line_non_comment_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    LazyFnParser::new(single_line_non_comment_statement)
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
#[deprecated]
pub fn single_line_statement<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    or_vec(vec![
        comment::comment(),
        dim::dim(),
        constant::constant_p().convert_to_fn(),
        built_ins::parse_built_in(),
        sub_call::sub_call_or_assignment(),
        statement_go_to(),
        statement_on_error_go_to(),
    ])
}

pub fn single_line_statement_p<R>() -> impl Parser<R, Output = Statement>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    LazyFnParser::new(single_line_statement)
}

pub fn statement_label<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(and(name::bare_name(), read(':')), |(l, _)| {
        Statement::Label(l)
    })
}

pub fn statement_go_to<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(
        crate::parser::pc::ws::seq2(
            keyword(Keyword::GoTo),
            demand(
                name::bare_name(),
                QError::syntax_error_fn("Expected: label"),
            ),
            QError::syntax_error_fn("Expected: whitespace"),
            "statement_go_to",
        ),
        |(_, l)| Statement::GoTo(l),
    )
}

pub fn statement_on_error_go_to<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(
        crate::parser::pc::ws::seq4(
            keyword(Keyword::On),
            demand(
                keyword(Keyword::Error),
                QError::syntax_error_fn("Expected: ERROR"),
            ),
            demand(
                keyword(Keyword::GoTo),
                QError::syntax_error_fn("Expected: GOTO"),
            ),
            demand(
                name::bare_name(),
                QError::syntax_error_fn("Expected: label"),
            ),
            QError::syntax_error_fn_fn("Expected: whitespace"),
            "statement_on_error_go_to",
        ),
        |(_, _, _, l)| Statement::ErrorHandler(l),
    )
}

pub fn statement_illegal_keywords<R>() -> Box<dyn Fn(R) -> ReaderResult<R, Statement, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    or(
        and_then(keyword(Keyword::Wend), |_| Err(QError::WendWithoutWhile)),
        and_then(keyword(Keyword::Else), |_| Err(QError::ElseWithoutIf)),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{PrintNode, Statement, TopLevelToken};

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
