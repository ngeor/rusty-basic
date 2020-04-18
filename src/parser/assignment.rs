use super::{NameNode, Parser, ParserError, StatementNode};
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn read_demand_assignment_skipping_whitespace(
        &mut self,
        left_side: NameNode,
    ) -> Result<StatementNode, ParserError> {
        let right_side = self.read_demand_expression_skipping_whitespace()?;
        self.read_demand_eol_or_eof_skipping_whitespace()?;
        Ok(StatementNode::Assignment(left_side, right_side))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::common::Location;
    use crate::lexer::LexemeNode;
    use crate::parser::StatementNode;

    macro_rules! assert_top_level_assignment {
        ($input:expr, $name:expr, $value:expr) => {
            match parse($input).demand_single_statement() {
                StatementNode::Assignment(n, v) => {
                    assert_eq!(&n, $name);
                    assert_eq!(v, $value);
                }
                _ => panic!("expected assignment"),
            }
        };
    }

    #[test]
    fn test_numeric_assignment() {
        assert_top_level_assignment!("A = 42", "A", 42);
    }

    #[test]
    fn test_numeric_assignment_to_keyword_not_allowed() {
        assert_eq!(
            parse_err("FOR = 42"),
            ParserError::Unexpected(
                "Expected FOR counter variable".to_string(),
                LexemeNode::Symbol('=', Location::new(1, 5))
            )
        );
    }

    #[test]
    fn test_numeric_assignment_to_keyword_plus_number_allowed() {
        assert_top_level_assignment!("FOR42 = 42", "FOR42", 42);
    }
}
