use crate::built_ins;
use crate::common::*;
use crate::lexer::{BufLexer, Keyword, LexemeNode};
use crate::parser::assignment;
use crate::parser::buf_lexer::*;
use crate::parser::comment;
use crate::parser::constant;
use crate::parser::dim_parser;
use crate::parser::error::ParserError;
use crate::parser::for_loop;
use crate::parser::if_block;
use crate::parser::name;
use crate::parser::select_case;
use crate::parser::sub_call;
use crate::parser::types::*;
use crate::parser::while_wend::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    dim_parser::try_read(lexer)
        .or_try_read(|| constant::try_read(lexer))
        .or_try_read(|| comment::try_read(lexer))
        .or_try_read(|| built_ins::try_read(lexer))
        .or_try_read(|| sub_call::try_read(lexer))
        .or_try_read(|| assignment::try_read(lexer))
        .or_try_read(|| try_read_label(lexer))
        .or_try_read(|| if_block::try_read(lexer))
        .or_try_read(|| for_loop::try_read(lexer))
        .or_try_read(|| select_case::try_read(lexer))
        .or_try_read(|| try_older(lexer))
}

fn try_read_label<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<StatementNode>, ParserError> {
    in_transaction(lexer, do_read_label)
}

fn do_read_label<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<StatementNode, ParserError> {
    let Locatable {
        element: bare_name,
        pos,
    } = demand(lexer, name::try_read_bare, "Expected bare name")?;
    read_symbol(lexer, ':')?;
    Ok(Statement::Label(bare_name).at(pos))
}

// TODO migrate these remaining older style
fn try_older<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    let next = lexer.read()?;
    match next {
        LexemeNode::Keyword(Keyword::GoTo, _, pos) => demand_go_to(lexer).map(|x| Some(x.at(pos))),
        LexemeNode::Keyword(Keyword::On, _, pos) => demand_on(lexer).map(|x| Some(x.at(pos))),
        LexemeNode::Keyword(Keyword::While, _, pos) => {
            demand_while_block(lexer).map(|x| Some(x.at(pos)))
        }
        _ => Ok(None),
    }
}

fn demand_on<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Statement, ParserError> {
    read_demand_whitespace(lexer, "Expected space after ON")?;
    read_demand_keyword(lexer, Keyword::Error)?;
    read_demand_whitespace(lexer, "Expected space after ERROR")?;
    read_demand_keyword(lexer, Keyword::GoTo)?;
    read_demand_whitespace(lexer, "Expected space after GOTO")?;
    let name_node = demand(lexer, name::try_read_bare, "Expected label name")?;
    let Locatable { element: name, .. } = name_node;
    Ok(Statement::ErrorHandler(name))
}

fn demand_go_to<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Statement, ParserError> {
    read_demand_whitespace(lexer, "Expected space after GOTO")?;
    let name_node = demand(lexer, name::try_read_bare, "Expected label name")?;
    let Locatable { element: name, .. } = name_node;
    Ok(Statement::GoTo(name))
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
}
