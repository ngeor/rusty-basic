use crate::built_ins;
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

pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, QErrorNode> {
    dim_parser::take_if_dim()(lexer)
        .transpose()
        .or_try_read(|| constant::take_if_const()(lexer).transpose())
        .or_try_read(|| comment::take_if_comment()(lexer).transpose())
        .or_try_read(|| built_ins::take_if_built_in()(lexer).transpose())
        .or_try_read(|| sub_call::take_if_sub_call()(lexer).transpose())
        .or_try_read(|| assignment::take_if_assignment()(lexer).transpose())
        .or_try_read(|| take_if_label()(lexer).transpose())
        .or_try_read(|| if_block::try_read(lexer))
        .or_try_read(|| for_loop::try_read(lexer))
        .or_try_read(|| select_case::try_read(lexer))
        .or_try_read(|| while_wend::try_read(lexer))
        .or_try_read(|| take_if_go_to()(lexer).transpose())
        .or_try_read(|| try_read_on(lexer))
        .or_try_read(|| try_read_illegal_keywords(lexer))
}

fn try_read_illegal_keywords<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, QErrorNode> {
    let p = lexer.peek_ref_ng()?;
    if p.is_keyword(Keyword::Wend) {
        Err(QError::WendWithoutWhile).with_err_at(p.unwrap())
    } else if p.is_keyword(Keyword::Else) {
        Err(QError::ElseWithoutIf).with_err_at(p.unwrap())
    } else {
        Ok(None)
    }
}

fn take_if_label<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(in_transaction_pc(apply(
        |(l, _)| l.map(|n| Statement::Label(n)),
        and(name::take_if_bare_name_node(), take_if_symbol(':')),
    )))
}

fn try_read_on<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, QErrorNode> {
    if !lexer.peek_ref_ng().is_keyword(Keyword::On) {
        return Ok(None);
    }
    let pos = lexer.read()?.pos();
    read_whitespace(lexer, "Expected space after ON")?;
    read_keyword(lexer, Keyword::Error)?;
    read_whitespace(lexer, "Expected space after ERROR")?;
    read_keyword(lexer, Keyword::GoTo)?;
    read_whitespace(lexer, "Expected space after GOTO")?;
    let name_node = read(lexer, name::try_read_bare, "Expected label name")?;
    let Locatable { element: name, .. } = name_node;
    Ok(Some(Statement::ErrorHandler(name).at(pos)))
}

fn take_if_go_to<T: BufRead + 'static>() -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    Box::new(apply(
        |(l, r)| Statement::GoTo(r.strip_location()).at(l.pos()),
        with_whitespace_between(
            take_if_keyword(Keyword::GoTo),
            name::take_if_bare_name_node(),
        ),
    ))
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
