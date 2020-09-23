use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::param_name;
use crate::parser::pc::common::*;
use crate::parser::pc::map::{and_then_none, map};
use crate::parser::pc::*;
use crate::parser::pc_specific::*;
use crate::parser::types::*;
use std::io::BufRead;

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

pub fn declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    drop_left(crate::parser::pc::ws::seq2(
        keyword(Keyword::Declare),
        demand(
            or(function_declaration_token(), sub_declaration_token()),
            QError::syntax_error_fn("Expected: FUNCTION or SUB after DECLARE"),
        ),
        QError::syntax_error_fn("Expected: whitespace after DECLARE"),
    ))
}

fn function_declaration_token<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(function_declaration(), |(n, p)| {
        TopLevelToken::FunctionDeclaration(n, p)
    })
}

pub fn function_declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (NameNode, ParamNameNodes), QError>> {
    map(
        seq5(
            keyword(Keyword::Function),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after FUNCTION"),
            ),
            demand(
                name::name_node(),
                QError::syntax_error_fn("Expected: function name"),
            ),
            crate::parser::pc::ws::zero_or_more(),
            opt_declaration_parameters(),
        ),
        |(_, _, n, _, opt_p)| (n, opt_p),
    )
}

fn sub_declaration_token<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, TopLevelToken, QError>> {
    map(sub_declaration(), |(n, p)| {
        TopLevelToken::SubDeclaration(n, p)
    })
}

pub fn sub_declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, (BareNameNode, ParamNameNodes), QError>> {
    map(
        seq5(
            keyword(Keyword::Sub),
            demand(
                crate::parser::pc::ws::one_or_more(),
                QError::syntax_error_fn("Expected: whitespace after SUB"),
            ),
            demand(
                name::bare_name_node(),
                QError::syntax_error_fn("Expected: sub name"),
            ),
            crate::parser::pc::ws::zero_or_more(),
            opt_declaration_parameters(),
        ),
        |(_, _, n, _, opt_p)| (n, opt_p),
    )
}

fn opt_declaration_parameters<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ParamNameNodes, QError>> {
    and_then_none(
        in_parenthesis(csv_zero_or_more(param_name::param_name_node())),
        || Ok(vec![]),
    )
}

#[cfg(test)]
mod tests {
    use crate::common::*;
    use crate::parser::test_utils::*;
    use crate::parser::{Name, ParamName, Statement, TopLevelToken, TypeQualifier};

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
            vec![ParamName::Compact("N".into(), TypeQualifier::BangSingle)]
        );
    }

    #[test]
    fn test_lower_case() {
        assert_function_declaration!(
            "declare function echo$(msg$)",
            Name::from("echo$"),
            vec![ParamName::Compact("msg".into(), TypeQualifier::DollarString)]
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
                    vec![ParamName::Bare("X".into()).at_rc(2, 31)]
                )
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Echoes stuff back".to_string()))
                    .at_rc(2, 34),
                TopLevelToken::FunctionImplementation(
                    "Echo".as_name(3, 18),
                    vec![ParamName::Bare("X".into()).at_rc(3, 23)],
                    vec![Statement::Comment(" Implementation of Echo".to_string()).at_rc(3, 26)]
                )
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
}
