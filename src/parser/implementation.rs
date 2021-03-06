use crate::common::*;
use crate::parser::declaration;
use crate::parser::pc::*;
use crate::parser::pc_specific::{demand_keyword_pair_p, keyword_p};
use crate::parser::statements;
use crate::parser::types::*;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation_p<R>() -> impl Parser<R, Output = TopLevelToken>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    function_implementation_p().or(sub_implementation_p())
}

fn function_implementation_p<R>() -> impl Parser<R, Output = TopLevelToken>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    static_declaration_p(declaration::function_declaration_p())
        .and_demand(statements::zero_or_more_statements_p(keyword_p(
            Keyword::End,
        )))
        .and_demand(demand_keyword_pair_p(Keyword::End, Keyword::Function))
        .keep_left()
        .map(|(((name, params), is_static), body)| {
            TopLevelToken::FunctionImplementation(FunctionImplementation {
                name,
                params,
                body,
                is_static,
            })
        })
}

fn sub_implementation_p<R>() -> impl Parser<R, Output = TopLevelToken>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    static_declaration_p(declaration::sub_declaration_p())
        .and_demand(statements::zero_or_more_statements_p(keyword_p(
            Keyword::End,
        )))
        .and_demand(demand_keyword_pair_p(Keyword::End, Keyword::Sub))
        .keep_left()
        .map(|(((name, params), is_static), body)| {
            TopLevelToken::SubImplementation(SubImplementation {
                name,
                params,
                body,
                is_static,
            })
        })
}

fn static_declaration_p<R, P, T>(parser: P) -> impl Parser<R, Output = (T, bool)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
    P: Parser<R, Output = T> + 'static,
{
    parser
        .and_opt(keyword_p(Keyword::Static).preceded_by_opt_ws())
        .map(|(l, r)| (l, r.is_some()))
}

#[cfg(test)]
mod tests {
    use crate::assert_parser_err;
    use crate::parser::test_utils::*;

    use super::*;

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
                    Expression::var_unresolved("Add"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("A".as_var_expr(3, 19)),
                        Box::new("B".as_var_expr(3, 23)),
                        ExpressionType::Unresolved
                    )
                    .at(Location::new(3, 21))
                )
                .at_rc(3, 13)],
                is_static: false
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
                    Expression::var_unresolved("add"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("a".as_var_expr(3, 19)),
                        Box::new("b".as_var_expr(3, 23)),
                        ExpressionType::Unresolved
                    )
                    .at_rc(3, 21)
                )
                .at_rc(3, 13)],
                is_static: false
            })
            .at_rc(2, 9)
        );
    }

    #[test]
    fn test_string_fixed_length_function_param_not_allowed() {
        let input = "
        FUNCTION Echo(X AS STRING * 5)
        END FUNCTION";
        assert_parser_err!(input, QError::syntax_error("Expected: closing parenthesis"));
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "
        SUB Echo(X AS STRING * 5)
        END SUB";
        assert_parser_err!(input, QError::syntax_error("Expected: closing parenthesis"));
    }

    #[test]
    fn test_user_defined_function_param_cannot_include_period() {
        let input = "
        FUNCTION Echo(X.Y AS Card)
        END FUNCTION
        ";
        assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
    }

    #[test]
    fn test_user_defined_sub_param_cannot_include_period() {
        let input = "
        SUB Echo(X.Y AS Card)
        END SUB
        ";
        assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
    }
}
