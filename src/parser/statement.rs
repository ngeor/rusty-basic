use crate::built_ins;
use crate::char_reader::*;
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::assignment;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim_parser;
use crate::parser::for_loop;
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend;
use std::io::BufRead;

pub fn statement_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    with_pos(statement())
}

pub fn statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    or_vec_ng(vec![
        dim_parser::dim(),
        constant::constant(),
        comment::comment(),
        built_ins::parse_built_in(),
        sub_call::sub_call(),
        assignment::assignment(),
        statement_label(),
        statement_if_block(),
        statement_for_loop(),
        statement_select_case(),
        statement_while_wend(),
        statement_go_to(),
        statement_on_error_go_to(),
        statement_illegal_keywords(),
    ])
}

pub fn statement_label<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(and_ng(name::bare_name(), try_read_char(':')), |(l, _)| {
        Statement::Label(l)
    })
}

pub fn statement_if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_for_loop<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_while_wend<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(with_keyword(Keyword::GoTo, name::bare_name()), |l| {
        Statement::GoTo(l)
    })
}

pub fn statement_on_error_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(
        with_two_keywords(
            Keyword::On,
            Keyword::Error,
            with_keyword(Keyword::GoTo, name::bare_name()),
        ),
        |l| Statement::ErrorHandler(l),
    )
}

pub fn statement_illegal_keywords<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    or_ng(
        map_to_result_no_undo(with_pos(try_read_keyword(Keyword::Wend)), |k| {
            Err(QError::WendWithoutWhile).with_err_at(k)
        }),
        map_to_result_no_undo(with_pos(try_read_keyword(Keyword::Else)), |k| {
            Err(QError::ElseWithoutIf).with_err_at(k)
        }),
    )
}

#[deprecated]
pub fn take_if_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    or_vec(vec![
        dim_parser::take_if_dim(),
        constant::take_if_const(),
        comment::take_if_comment(),
        built_ins::take_if_built_in(),
        sub_call::take_if_sub_call(),
        assignment::take_if_assignment(),
        take_if_label(),
        if_block::take_if_if_block(),
        for_loop::take_if_for_loop(),
        select_case::take_if_select_case(),
        while_wend::take_if_while_wend(),
        take_if_go_to(),
        take_if_on_error_goto(),
        try_read_illegal_keywords(),
    ])
}

#[deprecated]
fn try_read_illegal_keywords<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    or_vec(vec![
        Box::new(switch_err(
            |p| Some(Err(QError::WendWithoutWhile).with_err_at(p)),
            take_if_keyword(Keyword::Wend),
        )),
        Box::new(switch_err(
            |p| Some(Err(QError::ElseWithoutIf).with_err_at(p)),
            take_if_keyword(Keyword::Else),
        )),
    ])
}

#[deprecated]
fn take_if_label<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(in_transaction_pc(apply(
        |(l, _)| l.map(|n| Statement::Label(n)),
        and(name::take_if_bare_name_node(), take_if_symbol(':')),
    )))
}

#[deprecated]
fn take_if_on_error_goto<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |(l, (_, (_, r)))| Statement::ErrorHandler(r.strip_location()).at(l.pos()),
        with_whitespace_between(
            take_if_keyword(Keyword::On),
            with_whitespace_between(
                take_if_keyword(Keyword::Error),
                with_whitespace_between(
                    take_if_keyword(Keyword::GoTo),
                    name::take_if_bare_name_node(),
                ),
            ),
        ),
    )
}

#[deprecated]
fn take_if_go_to<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |(l, r)| Statement::GoTo(r.strip_location()).at(l.pos()),
        with_whitespace_between(
            take_if_keyword(Keyword::GoTo),
            name::take_if_bare_name_node(),
        ),
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
