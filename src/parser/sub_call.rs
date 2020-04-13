use super::{ExpressionNode, NameNode, Parser, StatementNode};
use crate::lexer::{LexemeNode, LexerError};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn try_parse_sub_call(&mut self) -> Result<Option<StatementNode>, LexerError> {
        match self.buf_lexer.read_ref()? {
            LexemeNode::Word(_, _) => Ok(Some(self._parse_sub_call()?)),
            _ => Ok(None),
        }
    }

    fn _parse_sub_call(&mut self) -> Result<StatementNode, LexerError> {
        let (sub_name, pos) = self.buf_lexer.demand_any_word()?;
        let found_whitespace = self.buf_lexer.skip_whitespace()?;
        let args: Vec<ExpressionNode> = if found_whitespace {
            self.parse_expression_list()?
        } else {
            vec![]
        };
        self.buf_lexer.demand_eol_or_eof()?;
        Ok(StatementNode::SubCall(
            NameNode::from(sub_name, None, pos),
            args,
        ))
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
