use crate::common::*;
use crate::parser::declaration;
use crate::parser::pc::common::*;
use crate::parser::pc::map::map;
use crate::parser::pc::*;
use crate::parser::pc2::Parser;
use crate::parser::pc_specific::*;
use crate::parser::statements;
use crate::parser::types::*;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    or(function_implementation(), sub_implementation())
}

pub fn function_implementation<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(
        seq5(
            declaration::function_declaration_p().convert_to_fn(),
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
        |((name, params), body, _, _, _)| {
            TopLevelToken::FunctionImplementation(FunctionImplementation { name, params, body })
        },
    )
}

pub fn sub_implementation<R>() -> Box<dyn Fn(R) -> ReaderResult<R, TopLevelToken, QError>>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    map(
        seq5(
            declaration::sub_declaration_p().convert_to_fn(),
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
        |((name, params), body, _, _, _)| {
            TopLevelToken::SubImplementation(SubImplementation { name, params, body })
        },
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
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name: "Add".as_name(2, 18),
                params: vec![
                    ParamName::new("A".into(), ParamType::Bare).at_rc(2, 22),
                    ParamName::new("B".into(), ParamType::Bare).at_rc(2, 25)
                ],
                body: vec![Statement::Assignment(
                    Expression::var("Add"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("A".as_var_expr(3, 19)),
                        Box::new("B".as_var_expr(3, 23)),
                        ExpressionType::Unresolved
                    )
                    .at(Location::new(3, 21))
                )
                .at_rc(3, 13)]
            })
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
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name: "add".as_name(2, 18),
                params: vec![
                    ParamName::new("a".into(), ParamType::Bare).at_rc(2, 22),
                    ParamName::new("b".into(), ParamType::Bare).at_rc(2, 25)
                ],
                body: vec![Statement::Assignment(
                    Expression::var("add"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("a".as_var_expr(3, 19)),
                        Box::new("b".as_var_expr(3, 23)),
                        ExpressionType::Unresolved
                    )
                    .at_rc(3, 21)
                )
                .at_rc(3, 13)]
            })
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

    #[test]
    fn test_user_defined_function_param_cannot_include_period() {
        let input = "
        FUNCTION Echo(X.Y AS Card)
        END FUNCTION
        ";
        assert_eq!(parse_err(input), QError::IdentifierCannotIncludePeriod);
    }

    #[test]
    fn test_user_defined_sub_param_cannot_include_period() {
        let input = "
        SUB Echo(X.Y AS Card)
        END SUB
        ";
        assert_eq!(parse_err(input), QError::IdentifierCannotIncludePeriod);
    }
}
