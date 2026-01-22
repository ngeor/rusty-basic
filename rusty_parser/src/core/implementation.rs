use rusty_pc::and::{IgnoringBothCombiner, opt_and};
use rusty_pc::*;

use crate::core::declaration::{function_declaration_p, sub_declaration_p};
use crate::core::statements::zero_or_more_statements;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{ParserError, *};

// FunctionImplementation ::= <FunctionDeclaration> eol <Statements> eol END<ws+>FUNCTION
// SubImplementation      ::= <SubDeclaration> eol <Statements> eol END<ws+>SUB

pub fn implementation_p() -> impl Parser<StringView, Output = GlobalStatement, Error = ParserError>
{
    function_implementation_p().or(sub_implementation_p())
}

fn function_implementation_p()
-> impl Parser<StringView, Output = GlobalStatement, Error = ParserError> {
    seq3(
        static_declaration_p(function_declaration_p()),
        zero_or_more_statements!(Keyword::End),
        keyword_pair(Keyword::End, Keyword::Function),
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

fn sub_implementation_p() -> impl Parser<StringView, Output = GlobalStatement, Error = ParserError>
{
    seq3(
        static_declaration_p(sub_declaration_p()),
        zero_or_more_statements!(Keyword::End),
        keyword_pair(Keyword::End, Keyword::Sub),
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

fn static_declaration_p<P, T>(
    parser: P,
) -> impl Parser<StringView, Output = (T, bool), Error = ParserError>
where
    P: Parser<StringView, Output = T, Error = ParserError>,
{
    parser.and_opt(
        opt_and(
            whitespace_ignoring(),
            keyword(Keyword::Static),
            IgnoringBothCombiner,
        ),
        |l, r: Option<()>| (l, r.is_some()),
    )
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use super::*;
    use crate::assert_parser_err;
    use crate::test_utils::*;

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
                body: vec![
                    Statement::assignment(
                        Expression::var_unresolved("Add"),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("A".as_var_expr(3, 19)),
                            Box::new("B".as_var_expr(3, 23)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(3, 21)
                    )
                    .at_rc(3, 13)
                ],
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
                body: vec![
                    Statement::assignment(
                        Expression::var_unresolved("add"),
                        Expression::BinaryExpression(
                            Operator::Plus,
                            Box::new("a".as_var_expr(3, 19)),
                            Box::new("b".as_var_expr(3, 23)),
                            ExpressionType::Unresolved
                        )
                        .at_rc(3, 21)
                    )
                    .at_rc(3, 13)
                ],
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
        assert_parser_err!(input, expected(")"));
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "
        SUB Echo(X AS STRING * 5)
        END SUB";
        assert_parser_err!(input, expected(")"));
    }

    #[test]
    fn test_user_defined_function_param_cannot_include_period() {
        let input = "
        FUNCTION Echo(X.Y AS Card)
        END FUNCTION
        ";
        // TODO should also be reported as IdentifierCannotIncludePeriod
        assert_parser_err!(
            input,
            "Expected: DOUBLE or INTEGER or LONG or SINGLE or STRING"
        );
    }

    #[test]
    fn test_user_defined_sub_param_cannot_include_period() {
        let input = "
        SUB Echo(X.Y AS Card)
        END SUB
        ";
        // TODO should also be reported as IdentifierCannotIncludePeriod
        assert_parser_err!(
            input,
            "Expected: DOUBLE or INTEGER or LONG or SINGLE or STRING"
        );
    }
}
