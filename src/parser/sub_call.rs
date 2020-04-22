use super::{unexpected, BareNameNode, ExpressionNode, Parser, ParserError, StatementNode};
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
        Ok(StatementNode::SubCall(name_node, args))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::common::Location;
    use crate::parser::{Operand, OperandNode, TopLevelTokenNode};

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall("PRINT".as_bare_name(1, 1), vec![])
        );
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall(
                "PRINT".as_bare_name(1, 1),
                vec!["Hello, world!".as_lit_expr(1, 7)]
            )
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS").demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall(
                "PRINT".as_bare_name(1, 1),
                vec!["Hello, world!".as_lit_expr(1, 7)]
            )
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS").demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall(
                "PRINT".as_bare_name(1, 1),
                vec!["Hello".as_lit_expr(1, 7), "world!".as_lit_expr(1, 16)]
            )
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS");
        assert_eq!(
            program,
            vec![
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_bare_name(1, 1),
                    vec!["Hello, world!".as_lit_expr(1, 7)]
                )),
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "SYSTEM".as_bare_name(2, 1),
                    vec![]
                )),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS");
        assert_eq!(
            program,
            vec![
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "INPUT".as_bare_name(1, 1),
                    vec!["N".as_var_expr(1, 7)]
                )),
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_bare_name(2, 1),
                    vec!["N".as_var_expr(2, 7)]
                )),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_environ() {
        let program = parse_file("ENVIRON.BAS");
        assert_eq!(
            program,
            vec![TopLevelTokenNode::Statement(StatementNode::SubCall(
                "PRINT".as_bare_name(1, 1),
                vec![ExpressionNode::FunctionCall(
                    "ENVIRON$".as_name(1, 7),
                    vec!["PATH".as_lit_expr(1, 16)]
                )]
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
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelTokenNode::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec![],
                    Location::new(2, 9)
                ),
                // Hello
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "Hello".as_bare_name(3, 9),
                    vec![]
                )),
                // SUB Hello
                TopLevelTokenNode::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec![],
                    vec![StatementNode::SubCall(
                        "ENVIRON".as_bare_name(5, 13),
                        vec!["FOO=BAR".as_lit_expr(5, 21)]
                    )],
                    Location::new(4, 9)
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
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                // DECLARE SUB Hello
                TopLevelTokenNode::SubDeclaration(
                    "Hello".as_bare_name(2, 21),
                    vec!["N$".as_name(2, 27), "V$".as_name(2, 31)],
                    Location::new(2, 9)
                ),
                // Hello
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "Hello".as_bare_name(3, 9),
                    vec!["FOO".as_lit_expr(3, 15), "BAR".as_lit_expr(3, 22)]
                )),
                // SUB Hello
                TopLevelTokenNode::SubImplementation(
                    "Hello".as_bare_name(4, 13),
                    vec!["N$".as_name(4, 19), "V$".as_name(4, 23)],
                    vec![StatementNode::SubCall(
                        "ENVIRON".as_bare_name(5, 13),
                        vec![ExpressionNode::BinaryExpression(
                            OperandNode::new(Operand::Plus, Location::new(5, 24)),
                            Box::new("N$".as_var_expr(5, 21)),
                            Box::new(ExpressionNode::BinaryExpression(
                                OperandNode::new(Operand::Plus, Location::new(5, 30)),
                                Box::new("=".as_lit_expr(5, 26)),
                                Box::new("V$".as_var_expr(5, 32))
                            ))
                        )]
                    )],
                    Location::new(4, 9)
                )
            ]
        );
    }
}
