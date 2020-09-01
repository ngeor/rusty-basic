use crate::common::*;
use crate::parser::assignment;
use crate::parser::built_ins;
use crate::parser::char_reader::*;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim_parser;
use crate::parser::for_loop;
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::loc::*;
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::*;
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;
use std::io::BufRead;

pub fn statement_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, StatementNode, QError>> {
    with_pos(statement())
}

pub fn statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    or_vec(vec![
        dim_parser::dim(),
        constant::constant(),
        comment::comment(),
        built_ins::parse_built_in(),
        sub_call::sub_call(),
        assignment::assignment(),
        statement_label(),
        if_block::if_block(),
        for_loop::for_loop(),
        select_case::select_case(),
        while_wend::while_wend(),
        statement_go_to(),
        statement_on_error_go_to(),
        statement_illegal_keywords(),
    ])
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// excluding comments.
pub fn single_line_non_comment_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    or_vec(vec![
        dim_parser::dim(),
        constant::constant(),
        built_ins::parse_built_in(),
        sub_call::sub_call(),
        assignment::assignment(),
        statement_go_to(),
        statement_on_error_go_to(),
    ])
}

/// Tries to read a statement that is allowed to be on a single line IF statement,
/// including comments.
pub fn single_line_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    or_vec(vec![
        comment::comment(),
        dim_parser::dim(),
        constant::constant(),
        built_ins::parse_built_in(),
        sub_call::sub_call(),
        assignment::assignment(),
        statement_go_to(),
        statement_on_error_go_to(),
    ])
}

pub fn statement_label<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(and(name::bare_name(), try_read(':')), |(l, _)| {
        Statement::Label(l)
    })
}

pub fn statement_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        crate::parser::pc::ws::seq2(
            try_read_keyword(Keyword::GoTo),
            demand(
                name::bare_name(),
                QError::syntax_error_fn("Expected: label"),
            ),
            QError::syntax_error_fn("Expected: whitespace"),
        ),
        |(_, l)| Statement::GoTo(l),
    )
}

pub fn statement_on_error_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(
        crate::parser::pc::ws::seq4(
            try_read_keyword(Keyword::On),
            demand(
                try_read_keyword(Keyword::Error),
                QError::syntax_error_fn("Expected: ERROR"),
            ),
            demand(
                try_read_keyword(Keyword::GoTo),
                QError::syntax_error_fn("Expected: GOTO"),
            ),
            demand(
                name::bare_name(),
                QError::syntax_error_fn("Expected: label"),
            ),
            QError::syntax_error_fn_fn("Expected: whitespace"),
        ),
        |(_, _, _, l)| Statement::ErrorHandler(l),
    )
}

pub fn statement_illegal_keywords<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    or(
        and_then(try_read_keyword(Keyword::Wend), |_| {
            Err(QError::WendWithoutWhile)
        }),
        and_then(try_read_keyword(Keyword::Else), |_| {
            Err(QError::ElseWithoutIf)
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Statement, TopLevelToken};

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
            vec![TopLevelToken::Statement(Statement::SubCall(
                "PRINT".into(),
                vec![42.as_lit_expr(1, 9)]
            ))
            .at_rc(1, 3)]
        );
    }
}
