use super::{
    unexpected, BareNameNode, ExpressionNode, Parser, ParserError, Statement, StatementNode,
};
use crate::common::*;
use crate::lexer::LexemeNode;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_sub_call(
        &mut self,
        name_node: BareNameNode,
        initial: LexemeNode,
    ) -> Result<StatementNode, ParserError> {
        let mut args: Vec<ExpressionNode> = vec![];
        const STATE_INITIAL: u8 = 0;
        const STATE_EOL_OR_EOF: u8 = 1;
        const STATE_ARG: u8 = 2;
        const STATE_COMMA: u8 = 3;
        let mut state = STATE_INITIAL;
        let mut next = initial;
        while state != STATE_EOL_OR_EOF {
            match next {
                LexemeNode::EOF(_) | LexemeNode::EOL(_, _) => {
                    if state == STATE_INITIAL || state == STATE_ARG {
                        state = STATE_EOL_OR_EOF;
                    } else {
                        return unexpected("Expected argument after comma", next);
                    }
                }
                LexemeNode::Symbol(',', _) => {
                    if state == STATE_ARG {
                        state = STATE_COMMA;
                        next = self.read_skipping_whitespace()?;
                    } else {
                        return unexpected("Syntax error", next);
                    }
                }
                _ => {
                    if state == STATE_INITIAL || state == STATE_COMMA {
                        args.push(self.demand_expression(next)?);
                        state = STATE_ARG;
                        next = self.read_skipping_whitespace()?;
                    } else {
                        return unexpected("Expected comma or EOL", next);
                    }
                }
            }
        }
        let (bare_name, pos) = name_node.consume();
        Ok(Statement::SubCall(bare_name, args).at(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Name, Operand, Statement, TopLevelToken};

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input).demand_single_statement();
        assert_eq!(program, Statement::SubCall("PRINT".into(), vec![]));
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall("PRINT".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall("PRINT".into(), vec!["Hello, world!".as_lit_expr(1, 7)])
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS").demand_single_statement();
        assert_eq!(
            program,
            Statement::SubCall(
                "PRINT".into(),
                vec!["Hello".as_lit_expr(1, 7), "world!".as_lit_expr(1, 16)]
            )
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "PRINT".into(),
                    vec!["Hello, world!".as_lit_expr(1, 7)]
                )),
                TopLevelToken::Statement(Statement::SubCall("SYSTEM".into(), vec![])),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::SubCall(
                    "INPUT".into(),
                    vec!["N".as_var_expr(1, 7)]
                )),
                TopLevelToken::Statement(Statement::SubCall(
                    "PRINT".into(),
                    vec!["N".as_var_expr(2, 7)]
                )),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_environ() {
        let program = parse_file("ENVIRON.BAS").strip_location();
        assert_eq!(
            program,
            vec![TopLevelToken::Statement(Statement::SubCall(
                "PRINT".into(),
                vec![Expression::FunctionCall(
                    Name::from("ENVIRON$"),
                    vec!["PATH".as_lit_expr(1, 16)]
                )
                .at_rc(1, 7)]
            ))]
        );
    }

    #[test]
    fn test_parse_sub_call_user_defined_no_args() {
        let input = r#"
        DECLARE SUB Hello
        Hello
        SUB Hello
            ENVIRON "FOO=BAR"
        END SUB
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelToken::SubDeclaration("Hello".as_bare_name(2, 21), vec![],),
                // Hello
                TopLevelToken::Statement(Statement::SubCall("Hello".into(), vec![])),
                // SUB Hello
                TopLevelToken::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec![],
                    vec![
                        Statement::SubCall("ENVIRON".into(), vec!["FOO=BAR".as_lit_expr(5, 21)])
                            .at_rc(5, 13)
                    ],
                )
            ]
        );
    }

    #[test]
    fn test_parse_sub_call_user_defined_two_args() {
        let input = r#"
        DECLARE SUB Hello(N$, V$)
        Hello "FOO", "BAR"
        SUB Hello(N$, V$)
            ENVIRON N$ + "=" + V$
        END SUB
        "#;
        let program = parse(input).strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelToken::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec!["N$".as_name(2, 27), "V$".as_name(2, 31)],
                ),
                // Hello
                TopLevelToken::Statement(Statement::SubCall(
                    "Hello".into(),
                    vec!["FOO".as_lit_expr(3, 15), "BAR".as_lit_expr(3, 22)]
                )),
                // SUB Hello
                TopLevelToken::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec!["N$".as_name(4, 19), "V$".as_name(4, 23)],
                    vec![Statement::SubCall(
                        "ENVIRON".into(),
                        vec![Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new("N$".as_var_expr(5, 21)),
                            Box::new(
                                Expression::BinaryExpression(
                                    Operand::Plus,
                                    Box::new("=".as_lit_expr(5, 26)),
                                    Box::new("V$".as_var_expr(5, 32))
                                )
                                .at_rc(5, 30)
                            )
                        )
                        .at_rc(5, 24)]
                    )
                    .at_rc(5, 13)],
                )
            ]
        );
    }
}
