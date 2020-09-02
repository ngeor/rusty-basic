use crate::common::*;
use crate::parser::char_reader::EolReader;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::{and, drop_right, seq2};
use crate::parser::pc::copy::try_read;
use crate::parser::pc::map::map;
use crate::parser::pc::ws::zero_or_more_around;
use crate::parser::pc::ReaderResult;
use crate::parser::types::*;
use std::io::BufRead;

pub fn assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(assignment_tuple(), |(l, r)| Statement::Assignment(l, r))
}

/// Parses `<name> <ws*> = <ws*> <expression-node>`.
///
/// If the equals sign is read, the expression must be read.
///
/// Examples:
///
/// ```basic
/// A = 42
/// A$ = "hello" + ", world"
/// A%=1
/// ```
pub fn assignment_tuple<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (Name, ExpressionNode), QError>> {
    // not using seq3 in case it's not an assignment but a sub call
    seq2(
        drop_right(and(name::name(), zero_or_more_around(try_read('=')))),
        expression::demand_expression_node(),
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
                _ => panic!("Expected: assignment"),
            }
        };
    }

    #[test]
    fn test_numeric_assignment() {
        assert_top_level_assignment!("A = 42", "A", 42);
        assert_top_level_assignment!("B=1", "B", 1);
        assert_top_level_assignment!("CD =100", "CD", 100);
        assert_top_level_assignment!("E= 3", "E", 3);
    }

    #[test]
    fn test_numeric_assignment_to_keyword_not_allowed() {
        assert_eq!(
            parse_err("FOR = 42"),
            QError::SyntaxError("Expected: name after FOR".to_string())
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
