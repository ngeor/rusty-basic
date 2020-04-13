use super::{ExpressionNode, NameNode, Parser, StatementNode};
use crate::common::CaseInsensitiveString;
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
            NameNode::new(CaseInsensitiveString::new(sub_name), None, pos),
            args,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::parser::Expression;

    #[test]
    fn test_parse_sub_call_no_args() {
        let input = "PRINT";
        let program = parse(input);
        assert_eq!(program, vec![top_sub_call("PRINT", vec![])]);
    }

    #[test]
    fn test_parse_sub_call_single_arg_string_literal() {
        let input = "PRINT \"Hello, world!\"";
        let program = parse(input);
        assert_eq!(
            program,
            vec![top_sub_call(
                "PRINT",
                vec![Expression::from("Hello, world!")],
            )],
        );
    }

    #[test]
    fn test_parse_fixture_hello1() {
        let program = parse_file("HELLO1.BAS");
        assert_eq!(
            program,
            vec![top_sub_call(
                "PRINT",
                vec![Expression::from("Hello, world!")],
            )],
        );
    }

    #[test]
    fn test_parse_fixture_hello2() {
        let program = parse_file("HELLO2.BAS");
        assert_eq!(
            program,
            vec![top_sub_call(
                "PRINT",
                vec![Expression::from("Hello"), Expression::from("world!")],
            )],
        );
    }

    #[test]
    fn test_parse_fixture_hello_system() {
        let program = parse_file("HELLO_S.BAS");
        assert_eq!(
            program,
            vec![
                top_sub_call("PRINT", vec![Expression::from("Hello, world!")]),
                top_sub_call("SYSTEM", vec![]),
            ],
        );
    }

    #[test]
    fn test_parse_fixture_input() {
        let program = parse_file("INPUT.BAS");
        assert_eq!(
            program,
            vec![
                top_sub_call("INPUT", vec![Expression::variable_name("N")]),
                top_sub_call("PRINT", vec![Expression::variable_name("N")]),
            ],
        );
    }
}
