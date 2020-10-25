use crate::common::*;
use crate::parser::char_reader::EolReader;
use crate::parser::expression;
use crate::parser::name;
use crate::parser::pc::common::{and, drop_right, seq2};
use crate::parser::pc::map::map;
use crate::parser::pc::ws::zero_or_more_around;
use crate::parser::pc::{read, ReaderResult};
use crate::parser::types::*;
use std::io::BufRead;

pub fn assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Statement, QError>> {
    map(assignment_tuple(), |(l, r)| {
        Statement::Assignment(Expression::VariableName(l), r)
    })
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
        drop_right(and(name::name(), zero_or_more_around(read('=')))),
        expression::demand_expression_node(),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use super::*;
    use crate::parser::{Expression, TopLevelToken};

    macro_rules! assert_top_level_assignment {
        ($input:expr, $name:expr, $value:expr) => {
            match parse($input).demand_single_statement() {
                Statement::Assignment(n, crate::common::Locatable { element: v, .. }) => {
                    assert_eq!(n, Expression::var($name));
                    assert_eq!(v, Expression::IntegerLiteral($value));
                }
                _ => panic!("Expected: assignment"),
            }
        };
    }

    #[test]
    fn test_numeric_assignment() {
        let names = [
            "A",
            "BC",
            "A%",
            "A.B",
            "A..B",
            "A.B.",
            "C.%",
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLM", // longest identifier is 40 characters
            "ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLM%",
        ];
        let values = [1, -1, 0, 42];
        for name in &names {
            for value in &values {
                assert_top_level_assignment!(format!("{} = {}", name, value), *name, *value);
                assert_top_level_assignment!(format!("{}={}", name, value), *name, *value);
                assert_top_level_assignment!(format!("{} ={}", name, value), *name, *value);
                assert_top_level_assignment!(format!("{}= {}", name, value), *name, *value);
            }
        }
    }

    #[test]
    fn test_numeric_assignment_to_keyword_not_allowed() {
        assert_eq!(
            parse_err("FOR = 42"),
            QError::syntax_error("Expected: name after FOR")
        );
    }

    #[test]
    fn test_identifier_too_long() {
        assert_eq!(
            parse_err("ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLMN = 42"),
            QError::IdentifierTooLong
        );
        assert_eq!(
            parse_err("ABCDEFGHIJKLMNOPQRSTUVWXYZ.ABCDEFGHIJKLMN% = 42"),
            QError::IdentifierTooLong
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

    #[test]
    fn test_array_with_single_dimension() {
        let input = "A(2) = 1";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(Expression::var("A"), 1.as_lit_expr(1, 1))
        );
    }

    #[test]
    fn test_array_with_two_dimensions() {
        let input = "A(1, 2) = 3";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(Expression::var("A"), 1.as_lit_expr(1, 1))
        );
    }

    #[test]
    fn test_array_with_user_defined_type_element() {
        let input = "A(1).Height = 2";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(Expression::var("A"), 1.as_lit_expr(1, 1))
        );
    }
}
