use super::{ParserError, Statement, StatementNode};
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer::*;
use crate::parser::expression;
use crate::parser::name;
use std::io::BufRead;

pub fn try_read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Option<StatementNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Const) {
        return Ok(None);
    }
    let pos = lexer.read()?.location();
    read_demand_whitespace(lexer, "Expected whitespace after CONST")?;
    let name_node = demand(lexer, name::try_read, "Expected CONST name")?;
    skip_whitespace(lexer)?;
    read_symbol(lexer, '=')?;
    skip_whitespace(lexer)?;
    let right_side = demand(lexer, expression::try_read, "Expected CONST expression")?;
    Ok(Statement::Const(name_node, right_side).at(pos)).map(|x| Some(x))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Statement, TopLevelToken};

    #[test]
    fn parse_const() {
        let input = r#"
        CONST X = 42
        CONST Y$ = "hello"
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Const(
                    "X".as_name(2, 15),
                    42.as_lit_expr(2, 19),
                )),
                TopLevelToken::Statement(Statement::Const(
                    "Y$".as_name(3, 15),
                    "hello".as_lit_expr(3, 20),
                ))
            ]
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = "CONST ANSWER = 42 ' the answer to life, universe, everything";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Const(
                    "ANSWER".as_name(1, 7),
                    42.as_lit_expr(1, 16)
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 19)
            ]
        );
    }
}
