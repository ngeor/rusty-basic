#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::{AtRowCol, QError};
    use crate::parser::types::*;

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
                    Expression::var("ANSWER"),
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
            Statement::Assignment(
                Expression::func("A", vec![2.as_lit_expr(1, 3)]),
                1.as_lit_expr(1, 8)
            )
        );
    }

    #[test]
    fn test_array_with_two_dimensions() {
        let input = "A(1, 2) = 3";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::func("A", vec![1.as_lit_expr(1, 3), 2.as_lit_expr(1, 6)]),
                3.as_lit_expr(1, 11)
            )
        );
    }

    #[test]
    fn test_array_qualified() {
        let input = "A$(N!) = 1";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::func("A$", vec!["N!".as_var_expr(1, 4)]),
                1.as_lit_expr(1, 10)
            )
        );
    }

    #[test]
    fn test_array_with_user_defined_type_element() {
        let input = "A(1).Height = 2";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::Property(
                    Box::new(Expression::func("A", vec![1.as_lit_expr(1, 3)])),
                    "Height".into()
                ),
                2.as_lit_expr(1, 15)
            )
        );
    }

    #[test]
    fn test_unqualified_user_defined_type_element() {
        let input = "A.B = 2";
        let program = parse(input).demand_single_statement();
        assert_eq!(
            program,
            Statement::Assignment(
                Expression::Property(
                    Box::new(Expression::var("A")),
                    "B".into()
                ),
                2.as_lit_expr(1, 7)
            )
        );
    }
}
