use super::{Statement, StatementNode};
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::expression;
use crate::parser::name;
use std::io::BufRead;

pub fn take_if_assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<StatementNode>> {
    apply(
        |((name_node, _), right_side)| {
            name_node.map(|name| Statement::Assignment(name, right_side))
        },
        and(
            in_transaction_pc(and(
                name::take_if_name_node(),
                skipping_whitespace(take_if_symbol('=')),
            )),
            demand(
                "Expected expression",
                skipping_whitespace(expression::take_if_expression_node()),
            ),
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::{Expression, Name, TopLevelToken};

    macro_rules! assert_top_level_assignment {
        ($input:expr, $name:expr, $value:expr) => {
            match parse($input).demand_single_statement() {
                Statement::Assignment(n, crate::common::Locatable { element: v, .. }) => {
                    assert_eq!(n, Name::from($name));
                    assert_eq!(v, Expression::IntegerLiteral($value));
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
            QError::SyntaxError("Expected FOR counter variable".to_string())
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
