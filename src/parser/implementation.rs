use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declaration;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::statements;
use crate::parser::types::*;
use std::io::BufRead;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    or(function_implementation(), sub_implementation())
}

pub fn function_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(
        seq5(
            declaration::function_declaration(),
            statements::statements(
                keyword(Keyword::End),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
            demand(
                keyword(Keyword::End),
                QError::syntax_error_fn("Expected: END FUNCTION"),
            ),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after END"),
            ),
            demand(
                keyword(Keyword::Function),
                QError::syntax_error_fn("Expected: FUNCTION after END"),
            ),
        ),
        |((n, p), body, _, _, _)| TopLevelToken::FunctionImplementation(n, p, body),
    )
}

pub fn sub_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(
        seq5(
            declaration::sub_declaration(),
            statements::statements(
                keyword(Keyword::End),
                QError::syntax_error_fn("Expected: end-of-statement"),
            ),
            demand(
                keyword(Keyword::End),
                QError::syntax_error_fn("Expected: END SUB"),
            ),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after END"),
            ),
            demand(
                keyword(Keyword::Sub),
                QError::syntax_error_fn("Expected: SUB after END"),
            ),
        ),
        |((n, p), body, _, _, _)| TopLevelToken::SubImplementation(n, p, body),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::test_utils::*;

    #[test]
    fn test_function_implementation() {
        let input = "
        FUNCTION Add(A, B)
            Add = A + B
        END FUNCTION
        ";
        let result = parse(input).demand_single();
        assert_eq!(
            result,
            TopLevelToken::FunctionImplementation(
                "Add".as_name(2, 18),
                vec![
                    ParamName::new("A".into(), ParamType::Bare).at_rc(2, 22),
                    ParamName::new("B".into(), ParamType::Bare).at_rc(2, 25)
                ],
                vec![Statement::Assignment(
                    "Add".into(),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("A".as_var_expr(3, 19)),
                        Box::new("B".as_var_expr(3, 23))
                    )
                    .at(Location::new(3, 21))
                )
                .at_rc(3, 13)],
            )
            .at_rc(2, 9)
        );
    }

    #[test]
    fn test_function_implementation_lower_case() {
        let input = "
        function add(a, b)
            add = a + b
        end function
        ";
        let result = parse(input).demand_single();
        assert_eq!(
            result,
            TopLevelToken::FunctionImplementation(
                "add".as_name(2, 18),
                vec![
                    ParamName::new("a".into(), ParamType::Bare).at_rc(2, 22),
                    ParamName::new("b".into(), ParamType::Bare).at_rc(2, 25)
                ],
                vec![Statement::Assignment(
                    "add".into(),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("a".as_var_expr(3, 19)),
                        Box::new("b".as_var_expr(3, 23))
                    )
                    .at_rc(3, 21)
                )
                .at_rc(3, 13)],
            )
            .at_rc(2, 9)
        );
    }

    #[test]
    fn test_string_fixed_length_function_param_not_allowed() {
        let input = "
        FUNCTION Echo(X AS STRING * 5)
        END FUNCTION";
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Expected: closing parenthesis")
        );
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "
        SUB Echo(X AS STRING * 5)
        END SUB";
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Expected: closing parenthesis")
        );
    }
}
