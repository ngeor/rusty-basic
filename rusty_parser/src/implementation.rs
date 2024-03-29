use crate::declaration;
use crate::pc::*;
use crate::pc_specific::*;
use crate::statements::ZeroOrMoreStatements;
use crate::types::*;

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation_p() -> impl Parser<Output = GlobalStatement> {
    function_implementation_p().or(sub_implementation_p())
}

fn function_implementation_p() -> impl Parser<Output = GlobalStatement> {
    seq3(
        static_declaration_p(declaration::function_declaration_p()),
        ZeroOrMoreStatements::new(keyword(Keyword::End)),
        keyword_pair(Keyword::End, Keyword::Function).no_incomplete(),
        |((name, params), is_static), body, _| {
            GlobalStatement::FunctionImplementation(FunctionImplementation {
                name,
                params,
                body,
                is_static,
            })
        },
    )
}

fn sub_implementation_p() -> impl Parser<Output = GlobalStatement> {
    seq3(
        static_declaration_p(declaration::sub_declaration_p()),
        ZeroOrMoreStatements::new(keyword(Keyword::End)),
        keyword_pair(Keyword::End, Keyword::Sub).no_incomplete(),
        |((name, params), is_static), body, _| {
            GlobalStatement::SubImplementation(SubImplementation {
                name,
                params,
                body,
                is_static,
            })
        },
    )
}

fn static_declaration_p<P, T>(parser: P) -> impl Parser<Output = (T, bool)>
where
    P: Parser<Output = T> + 'static,
{
    parser
        .and_opt(OptAndPC::new(whitespace(), keyword(Keyword::Static)))
        .map(|(l, r)| (l, r.is_some()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_parser_err;
    use crate::test_utils::*;
    use crate::*;
    use rusty_common::*;

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
            GlobalStatement::FunctionImplementation(FunctionImplementation {
                name: "Add".as_name(2, 18),
                params: vec![
                    Parameter::new("A".into(), ParamType::Bare).at_rc(2, 22),
                    Parameter::new("B".into(), ParamType::Bare).at_rc(2, 25)
                ],
                body: vec![Statement::Assignment(
                    Expression::var_unresolved("Add"),
                    Expression::BinaryExpression(
                        Operator::Plus,
                        Box::new("A".as_var_expr(3, 19)),
                        Box::new("B".as_var_expr(3, 23)),
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
    fn test_function_implementation_lower_case() {
        let input = "
        function add(a, b)
            add = a + b
        end function
        ";
        let result = parse(input).demand_single();
        assert_eq!(
            result,
            GlobalStatement::FunctionImplementation(FunctionImplementation {
                name: "add".as_name(2, 18),
                params: vec![
                    Parameter::new("a".into(), ParamType::Bare).at_rc(2, 22),
                    Parameter::new("b".into(), ParamType::Bare).at_rc(2, 25)
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
        assert_parser_err!(input, ParseError::syntax_error("Expected: )"));
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "
        SUB Echo(X AS STRING * 5)
        END SUB";
        assert_parser_err!(input, ParseError::syntax_error("Expected: )"));
    }

    #[test]
    fn test_user_defined_function_param_cannot_include_period() {
        let input = "
        FUNCTION Echo(X.Y AS Card)
        END FUNCTION
        ";
        assert_parser_err!(
            input,
            "Expected: SINGLE or DOUBLE or STRING or INTEGER or LONG"
        );
    }

    #[test]
    fn test_user_defined_sub_param_cannot_include_period() {
        let input = "
        SUB Echo(X.Y AS Card)
        END SUB
        ";
        assert_parser_err!(
            input,
            "Expected: SINGLE or DOUBLE or STRING or INTEGER or LONG"
        );
    }
}
