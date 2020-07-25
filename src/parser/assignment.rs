use super::{NameNode, Parser, ParserError, Statement, StatementContext, StatementNode};
use crate::common::*;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn read_demand_assignment_skipping_whitespace(
        &mut self,
        left_side: NameNode,
        context: StatementContext,
    ) -> Result<StatementNode, ParserError> {
        let right_side = self.read_demand_expression_skipping_whitespace()?;
        // if multi-line, demand eol/eof -- otherwise, let the single-line if statement sort it out (might be ELSE following)
        self.finish_line(context)?;
        let (name, pos) = left_side.consume();
        Ok(Statement::Assignment(name, right_side).at(pos))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::lexer::LexemeNode;
    use crate::parser::{Expression, Name, TopLevelToken};

    macro_rules! assert_top_level_assignment {
        ($input:expr, $name:expr, $value:expr) => {
            match parse($input).demand_single_statement() {
                Statement::Assignment(n, v) => {
                    assert_eq!(n, Name::from($name));
                    assert_eq!(v.strip_location(), Expression::IntegerLiteral($value));
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

    #[test]
    fn test_inline_comment() {
        let input = "ANSWER = 42 ' the answer to life, universe, everything";
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                TopLevelToken::Statement(Statement::Assignment(
                    "ANSWER".into(),
                    42.as_lit_expr(1, 10)
                ))
                .at_rc(1, 1),
                TopLevelToken::Statement(Statement::Comment(
                    " the answer to life, universe, everything".to_string(),
                ))
                .at_rc(1, 13)
            ]
        );
    }
}
