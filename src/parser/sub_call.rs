use super::{unexpected, ExpressionNode, NameNode, Parser, ParserError, StatementNode};
use crate::lexer::LexemeNode;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn demand_sub_call(
        &mut self,
        name: NameNode,
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
        Ok(StatementNode::SubCall(name, args))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::TopLevelTokenNode;

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall("PRINT".as_name(1, 1), vec![])
        );
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            StatementNode::SubCall(
                "PRINT".as_name(1, 1),
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
                "PRINT".as_name(1, 1),
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
                "PRINT".as_name(1, 1),
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
                    "PRINT".as_name(1, 1),
                    vec!["Hello, world!".as_lit_expr(1, 7)]
                )),
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "SYSTEM".as_name(2, 1),
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
                    "INPUT".as_name(1, 1),
                    vec!["N".as_var_expr(1, 7)]
                )),
                TopLevelTokenNode::Statement(StatementNode::SubCall(
                    "PRINT".as_name(2, 1),
                    vec!["N".as_var_expr(2, 7)]
                )),
            ],
        );
    }
}
