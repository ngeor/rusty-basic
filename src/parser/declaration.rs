use crate::common::*;
use crate::parser::name;
use crate::parser::param_name::param_name_node_p;
use crate::parser::pc::*;
use crate::parser::pc_specific::{in_parenthesis_p, keyword_p, PcSpecific};
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

pub fn declaration_p<R>() -> impl Parser<R, Output = TopLevelToken>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Declare)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after DECLARE"))
        .and_demand(
            function_declaration_p()
                .map(|(n, p)| TopLevelToken::FunctionDeclaration(n, p))
                .or(sub_declaration_p().map(|(n, p)| TopLevelToken::SubDeclaration(n, p)))
                .or_syntax_error("Expected: FUNCTION or SUB after DECLARE"),
        )
        .keep_right()
}

pub fn function_declaration_p<R>() -> impl Parser<R, Output = (NameNode, ParamNameNodes)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Function)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after FUNCTION"))
        .and_demand(
            name::name_with_dot_p()
                .with_pos()
                .or_syntax_error("Expected: function name"),
        )
        .and_opt(whitespace_p())
        .and_opt(declaration_parameters_p())
        .map(|(((_, function_name_node), _), opt_p)| {
            (function_name_node, opt_p.unwrap_or_default())
        })
}

pub fn sub_declaration_p<R>() -> impl Parser<R, Output = (BareNameNode, ParamNameNodes)>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    keyword_p(Keyword::Sub)
        .and_demand(whitespace_p().or_syntax_error("Expected: whitespace after SUB"))
        .and_demand(
            name::bare_name_p()
                .with_pos()
                .or_syntax_error("Expected: sub name"),
        )
        .and_opt(whitespace_p())
        .and_opt(declaration_parameters_p())
        .map(|(((_, sub_name_node), _), opt_p)| (sub_name_node, opt_p.unwrap_or_default()))
}

fn declaration_parameters_p<R>() -> impl Parser<R, Output = ParamNameNodes>
where
    R: Reader<Item = char, Err = QError> + HasLocation + 'static,
{
    in_parenthesis_p(param_name_node_p().csv().map_none_to_default())
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{
        BuiltInStyle, FunctionImplementation, Name, ParamName, ParamType, Statement, TopLevelToken,
        TypeQualifier,
    };

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse($input).demand_single().strip_location() {
                TopLevelToken::FunctionDeclaration(name, parameters) => {
                    assert_eq!(name, $expected_function_name, "Function name mismatch");
                    assert_eq!(
                        parameters.len(),
                        $expected_params.len(),
                        "Parameter count mismatch"
                    );
                    let parameters_without_location: Vec<ParamName> =
                        parameters.into_iter().map(|x| x.strip_location()).collect();
                    for i in 0..parameters_without_location.len() {
                        assert_eq!(
                            parameters_without_location[i], $expected_params[i],
                            "Parameter {}",
                            i
                        );
                    }
                }
                _ => panic!(format!("{:?}", $input)),
            }
        };
    }

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
                    ]
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
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Expected: closing parenthesis")
        );
    }

    #[test]
    fn test_string_fixed_length_sub_param_not_allowed() {
        let input = "DECLARE SUB Echo(X AS STRING * 5)";
        assert_eq!(
            parse_err(input),
            QError::syntax_error("Expected: closing parenthesis")
        );
    }

    #[test]
    fn test_user_defined_function_param_cannot_include_period() {
        let input = "DECLARE FUNCTION Echo(X.Y AS Card)";
        assert_eq!(parse_err(input), QError::IdentifierCannotIncludePeriod);
    }

    #[test]
    fn test_user_defined_sub_param_cannot_include_period() {
        let input = "DECLARE SUB Echo(X.Y AS Card)";
        assert_eq!(parse_err(input), QError::IdentifierCannotIncludePeriod);
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
                    body: vec![]
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
