use crate::parser::name;
use crate::parser::param_name::param_name_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;

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

pub fn declaration_p() -> impl Parser<Output = TopLevelToken> {
    keyword_followed_by_whitespace_p(Keyword::Declare).then_demand(
        function_declaration_p()
            .map(|(n, p)| TopLevelToken::FunctionDeclaration(n, p))
            .or(sub_declaration_p().map(|(n, p)| TopLevelToken::SubDeclaration(n, p)))
            .or_syntax_error("Expected: FUNCTION or SUB after DECLARE"),
    )
}

pub fn function_declaration_p() -> impl Parser<Output = (NameNode, ParamNameNodes)> {
    seq4(
        keyword(Keyword::Function),
        whitespace().no_incomplete(),
        name::name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: function name"),
        declaration_parameters_p(),
        |_, _, function_name_node, declaration_parameters| {
            (function_name_node, declaration_parameters)
        },
    )
}

pub fn sub_declaration_p() -> impl Parser<Output = (BareNameNode, ParamNameNodes)> {
    seq4(
        keyword(Keyword::Sub),
        whitespace().no_incomplete(),
        name::bare_name_with_dots()
            .with_pos()
            .or_syntax_error("Expected: sub name"),
        declaration_parameters_p(),
        |_, _, sub_name_node, declaration_parameters| (sub_name_node, declaration_parameters),
    )
}

// result ::= "" | "(" ")" | "(" param_node (,param_node)* ")"
fn declaration_parameters_p() -> impl Parser<Output = ParamNameNodes> + NonOptParser {
    OptAndPC::new(
        whitespace(),
        in_parenthesis(csv(param_name_node_p()).allow_default()),
    )
    .keep_right()
    .allow_default()
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{
        BuiltInStyle, FunctionImplementation, Name, ParamName, ParamType, Statement, TopLevelToken,
        TypeQualifier,
    };
    use crate::{assert_function_declaration, assert_parser_err};

    #[test]
    fn test_fn() {
        assert_function_declaration!(
            "DECLARE FUNCTION Fib! (N!)",
            Name::from("Fib!"),
            vec![ParamName::new(
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
            vec![ParamName::new(
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
                TopLevelToken::FunctionDeclaration(
                    "Echo".as_name(2, 26),
                    vec![ParamName::new("X".into(), ParamType::Bare).at_rc(2, 31)]
                )
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Echoes stuff back".to_string()))
                    .at_rc(2, 34),
                TopLevelToken::FunctionImplementation(FunctionImplementation {
                    name: "Echo".as_name(3, 18),
                    params: vec![ParamName::new("X".into(), ParamType::Bare).at_rc(3, 23)],
                    body: vec![
                        Statement::Comment(" Implementation of Echo".to_string()).at_rc(3, 26)
                    ],
                    is_static: false
                })
                .at_rc(3, 9),
                TopLevelToken::Statement(Statement::Comment(" End of implementation".to_string()))
                    .at_rc(4, 22),
            ]
        );
    }

    #[test]
    fn test_string_fixed_length_function_param_not_allowed() {
        let input = "DECLARE FUNCTION Echo(X AS STRING * 5)";
        assert_parser_err!(input, QError::syntax_error("Expected: )"));
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "DECLARE SUB Echo(X AS STRING * 5)";
        assert_parser_err!(input, QError::syntax_error("Expected: )"));
    }

    #[test]
    fn test_user_defined_param_name_cannot_include_period() {
        let inputs = [
            "DECLARE FUNCTION Echo(X.Y AS Card)",
            "DECLARE SUB Echo(X.Y AS Card)",
        ];
        for input in inputs {
            assert_parser_err!(
                input,
                "Expected: SINGLE or DOUBLE or STRING or INTEGER or LONG"
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
            assert_parser_err!(input, QError::IdentifierCannotIncludePeriod);
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
                TopLevelToken::FunctionDeclaration(
                    "Echo".as_name(2, 26),
                    vec![ParamName::new(
                        "X".into(),
                        ParamType::Array(Box::new(ParamType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        )))
                    )
                    .at_rc(2, 31)]
                )
                .at_rc(2, 9),
                TopLevelToken::FunctionImplementation(FunctionImplementation {
                    name: "Echo".as_name(3, 18),
                    params: vec![ParamName::new(
                        "X".into(),
                        ParamType::Array(Box::new(ParamType::BuiltIn(
                            TypeQualifier::DollarString,
                            BuiltInStyle::Compact
                        )))
                    )
                    .at_rc(3, 23)],
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
            vec![TopLevelToken::SubDeclaration("ScrollUp".as_bare_name(2, 21), vec![]).at_rc(2, 9)]
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
            vec![TopLevelToken::SubDeclaration(
                "LCenter".as_bare_name(2, 21),
                vec![ParamName::new(
                    "text".into(),
                    ParamType::BuiltIn(TypeQualifier::DollarString, BuiltInStyle::Compact)
                )
                .at_rc(2, 30)]
            )
            .at_rc(2, 9)]
        );
    }
}
