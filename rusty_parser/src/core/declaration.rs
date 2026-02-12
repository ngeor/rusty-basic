use rusty_pc::and::opt_and_keep_right;
use rusty_pc::*;

use crate::core::name::{bare_name_p, name_p};
use crate::core::param_name::parameter_pos_p;
use crate::input::StringView;
use crate::pc_specific::*;
use crate::tokens::whitespace_ignoring;
use crate::{ParserError, *};

// Declaration           ::= DECLARE<ws+>(FunctionDeclaration|SubDeclaration)
// FunctionDeclaration   ::= FUNCTION<ws+><Name><ws*><DeclarationParameters>
// SubDeclaration        ::= SUB<ws+><BareName><ws*><DeclarationParameters>
// DeclarationParameters ::= <eof> | <eol> | '(' <DeclaredNames> ')'
// DeclaredNames         ::= <> | <DeclaredName> | <DeclaredName><ws*>,<ws*><DeclaredNames>
// DeclaredName          ::= <BareName> | <CompactBuiltIn> | <ExtendedBuiltIn> | <UserDefined>
// BareName              ::= [a-zA-Z]([a-zA-Z0-9\.]*) ! Keyword
// CompactBuiltIn        ::= <BareName>[!#$%&]
// ExtendedBuiltIn       ::= <BareName><ws+>AS<ws+>(SINGLE|DOUBLE|STRING|INTEGER|LONG)
// UserDefined           ::= <BareName><ws+>AS<ws+><BareName>

pub fn declaration_p() -> impl Parser<StringView, Output = GlobalStatement, Error = ParserError> {
    keyword_ws_p(Keyword::Declare).and_keep_right(
        OrParser::new(vec![
            Box::new(
                function_declaration_p().map(|(n, p)| GlobalStatement::function_declaration(n, p)),
            ),
            Box::new(sub_declaration_p().map(|(n, p)| GlobalStatement::sub_declaration(n, p))),
        ])
        .or_expected("FUNCTION or SUB after DECLARE"),
    )
}

pub fn function_declaration_p()
-> impl Parser<StringView, Output = (NamePos, Parameters), Error = ParserError> {
    seq3(
        keyword_ws_p(Keyword::Function),
        name_p().with_pos().or_expected("function name"),
        declaration_parameters_p(),
        |_, function_name_pos, declaration_parameters| (function_name_pos, declaration_parameters),
    )
}

pub fn sub_declaration_p()
-> impl Parser<StringView, Output = (BareNamePos, Parameters), Error = ParserError> {
    seq3(
        keyword_ws_p(Keyword::Sub),
        bare_name_p().with_pos().or_expected("sub name"),
        declaration_parameters_p(),
        |_, sub_name_pos, declaration_parameters| (sub_name_pos, declaration_parameters),
    )
}

// result ::= "" | "(" ")" | "(" parameter (,parameter)* ")"
fn declaration_parameters_p() -> impl Parser<StringView, Output = Parameters, Error = ParserError> {
    opt_and_keep_right(
        whitespace_ignoring(),
        in_parenthesis(csv(parameter_pos_p()).or_default()),
    )
    .or_default()
}

#[cfg(test)]
mod tests {
    use rusty_common::*;

    use crate::test_utils::*;
    use crate::{assert_function_declaration, assert_parser_err, *};

    #[test]
    fn test_fn() {
        assert_function_declaration!(
            "DECLARE FUNCTION Fib! (N!)",
            Name::from("Fib!"),
            vec![Parameter::new(
                "N".into(),
                ParamType::BuiltIn(TypeQualifier::BangSingle, BuiltInStyle::Compact)
            )]
        );
    }

    #[test]
    fn test_lower_case() {
        assert_function_declaration!(
            "declare function echo$(msg$)",
            Name::from("echo$"),
            vec![Parameter::new(
                "msg".into(),
                ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
            )]
        );
    }

    #[test]
    fn test_inline_comment() {
        let input = r#"
        DECLARE FUNCTION Echo(X) ' Echoes stuff back
        FUNCTION Echo(X) ' Implementation of Echo
        END FUNCTION ' End of implementation
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::function_declaration(
                    "Echo".as_name(2, 26),
                    vec![Parameter::new("X".into(), ParamType::Bare).at_rc(2, 31)]
                )
                .at_rc(2, 9),
                GlobalStatement::Statement(Statement::Comment(" Echoes stuff back".to_string()))
                    .at_rc(2, 34),
                GlobalStatement::FunctionImplementation(FunctionImplementation {
                    name: "Echo".as_name(3, 18),
                    params: vec![Parameter::new("X".into(), ParamType::Bare).at_rc(3, 23)],
                    body: vec![
                        Statement::Comment(" Implementation of Echo".to_string()).at_rc(3, 26)
                    ],
                    is_static: false
                })
                .at_rc(3, 9),
                GlobalStatement::Statement(Statement::Comment(
                    " End of implementation".to_string()
                ))
                .at_rc(4, 22),
            ]
        );
    }

    #[test]
    fn test_string_fixed_length_function_param_not_allowed() {
        let input = "DECLARE FUNCTION Echo(X AS STRING * 5)";
        assert_parser_err!(input, expected(")"));
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "DECLARE SUB Echo(X AS STRING * 5)";
        assert_parser_err!(input, expected(")"));
    }

    #[test]
    fn test_user_defined_param_name_cannot_include_period() {
        let inputs = [
            "DECLARE FUNCTION Echo(X.Y AS Card)",
            "DECLARE SUB Echo(X.Y AS Card)",
        ];
        for input in inputs {
            // TODO should also be reported as IdentifierCannotIncludePeriod
            assert_parser_err!(
                input,
                "Expected: DOUBLE or INTEGER or LONG or SINGLE or STRING"
            );
        }
    }

    #[test]
    fn test_user_defined_param_type_cannot_include_period() {
        let inputs = [
            "DECLARE FUNCTION Echo(XY AS Ca.rd)",
            "DECLARE SUB Echo(XY AS Ca.rd)",
        ];
        for input in inputs {
            assert_parser_err!(input, ParserError::IdentifierCannotIncludePeriod);
        }
    }

    #[test]
    fn test_array_parameter() {
        let input = r#"
        DECLARE FUNCTION Echo(X$())
        FUNCTION Echo(X$())
        END FUNCTION
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::function_declaration(
                    "Echo".as_name(2, 26),
                    vec![
                        Parameter::new(
                            "X".into(),
                            ParamType::Array(Box::new(ParamType::BuiltIn(
                                TypeQualifier::DollarString,
                                BuiltInStyle::Compact
                            )))
                        )
                        .at_rc(2, 31)
                    ]
                )
                .at_rc(2, 9),
                GlobalStatement::FunctionImplementation(FunctionImplementation {
                    name: "Echo".as_name(3, 18),
                    params: vec![
                        Parameter::new(
                            "X".into(),
                            ParamType::Array(Box::new(ParamType::BuiltIn(
                                TypeQualifier::DollarString,
                                BuiltInStyle::Compact
                            )))
                        )
                        .at_rc(3, 23)
                    ],
                    body: vec![],
                    is_static: false
                })
                .at_rc(3, 9),
            ]
        );
    }

    #[test]
    fn test_sub_no_args_space_after_sub_name() {
        let input = r#"
        DECLARE SUB ScrollUp ()
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::sub_declaration("ScrollUp".as_bare_name(2, 21), vec![])
                    .at_rc(2, 9)
            ]
        );
    }

    #[test]
    fn test_sub_one_arg_space_after_sub_name() {
        let input = r#"
        DECLARE SUB LCenter (text$)
        "#;
        let program = parse(input);
        assert_eq!(
            program,
            vec![
                GlobalStatement::sub_declaration(
                    "LCenter".as_bare_name(2, 21),
                    vec![
                        Parameter::new(
                            "text".into(),
                            ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                        )
                        .at_rc(2, 30)
                    ]
                )
                .at_rc(2, 9)
            ]
        );
    }
}
