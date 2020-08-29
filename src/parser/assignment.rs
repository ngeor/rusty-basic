use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::types::*;
use std::io::BufRead;

pub fn assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Statement, QError>)> {
    map(assignment_tuple(), |(l, r)| Statement::Assignment(l, r))
}

pub fn assignment_tuple<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<(Name, ExpressionNode), QError>)> {
    // not using seq3 in case it's not an assignment but a sub call
    map(
        and(
            name::name(),
            seq2(
                crate::parser::pc::ws::zero_or_more_leading(try_read('=')),
                demand(
                    crate::parser::pc::ws::zero_or_more_leading(expression::expression_node()),
                    QError::syntax_error_fn("Expected expression after ="),
                ),
            ),
        ),
        |(l, (_, r))| (l, r),
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
            QError::SyntaxError("Cannot parse after FOR".to_string())
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
