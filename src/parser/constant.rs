use super::{Statement, StatementNode};
use crate::char_reader::*;
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::name;
use std::io::BufRead;

pub fn constant<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QErrorNode>)> {
    map_ng(
        with_some_whitespace_between(
            try_read_keyword(Keyword::Const),
            with_any_whitespace_between(
                name::name_node(),
                with_any_whitespace_between(
                    try_read_char('='),
                    expression::expression_node(),
                    || QError::SyntaxError("Expected expression after =".to_string()),
                ),
                || QError::SyntaxError("Expected = after name".to_string()),
            ),
            || QError::SyntaxError("Expected name after CONST".to_string()),
        ),
        |(_, (n, (_, e)))| Statement::Const(n, e),
    )
}

#[deprecated]
pub fn take_if_const<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |(l, (name_node, (_, right_side)))| {
            let pos = l.pos();
            Statement::Const(name_node, right_side).at(pos)
        },
        with_whitespace_between(
            take_if_keyword(Keyword::Const),
            and(
                demand("Expected CONST name", name::take_if_name_node()),
                and(
                    demand(
                        "Expected = after CONST name",
                        skipping_whitespace(take_if_symbol('=')),
                    ),
                    demand(
                        "Expected CONST expression",
                        skipping_whitespace(expression::take_if_expression_node()),
                    ),
                ),
            ),
        ),
    )
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
