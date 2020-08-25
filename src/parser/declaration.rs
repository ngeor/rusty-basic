use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::declared_name;
use crate::parser::name;
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
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    with_keyword_before(
        Keyword::Declare,
        or(function_declaration_token(), sub_declaration_token()),
    )
}

fn function_declaration_token<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map(function_declaration(), |(n, p)| {
        TopLevelToken::FunctionDeclaration(n, p)
    })
}

pub fn function_declaration<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> (
        EolReader<T>,
        Result<(NameNode, DeclaredNameNodes), QErrorNode>,
    ),
> {
    map(
        with_keyword_before(
            Keyword::Function,
            if_first_maybe_second(
                name::name_node(),
                skipping_whitespace(declaration_parameters()),
            ),
        ),
        |(n, opt_p)| (n, opt_p.unwrap_or_default()),
    )
}

fn sub_declaration_token<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    map(sub_declaration(), |(n, p)| {
        TopLevelToken::SubDeclaration(n, p)
    })
}

pub fn sub_declaration<T: BufRead + 'static>() -> Box<
    dyn Fn(
        EolReader<T>,
    ) -> (
        EolReader<T>,
        Result<(BareNameNode, DeclaredNameNodes), QErrorNode>,
    ),
> {
    map(
        with_keyword_before(
            Keyword::Sub,
            if_first_maybe_second(
                name::bare_name_node(),
                skipping_whitespace(declaration_parameters()),
            ),
        ),
        |(n, opt_p)| (n, opt_p.unwrap_or_default()),
    )
}

fn declaration_parameters<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<DeclaredNameNodes, QErrorNode>)> {
    in_parenthesis(csv_zero_or_more(declared_name::declared_name_node()))
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{
        DeclaredName, Expression, Name, Operand, Statement, TopLevelToken, TypeQualifier,
    };

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse($input).demand_single().as_ref() {
                TopLevelToken::FunctionDeclaration(name, parameters) => {
                    assert_eq!(name, $expected_function_name, "Function name mismatch");
                    let x = $expected_params;
                    assert_eq!(parameters.len(), x.len(), "Parameter count mismatch");
                    for i in 0..x.len() {
                        assert_eq!(parameters[i].as_ref(), &x[i], "Parameter {}", i);
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
            &Name::from("Fib!"),
            vec![DeclaredName::compact("N", TypeQualifier::BangSingle)]
        );
    }

    #[test]
    fn test_lower_case() {
        assert_function_declaration!(
            "declare function echo$(msg$)",
            &Name::from("echo$"),
            vec![DeclaredName::compact("msg", TypeQualifier::DollarString)]
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
                    vec![DeclaredName::bare("X").at_rc(2, 31)]
                )
                .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Echoes stuff back".to_string()))
                    .at_rc(2, 34),
                TopLevelToken::FunctionImplementation(
                    "Echo".as_name(3, 18),
                    vec![DeclaredName::bare("X").at_rc(3, 23)],
                    vec![Statement::Comment(" Implementation of Echo".to_string()).at_rc(3, 26)]
                )
                .at_rc(3, 9),
                TopLevelToken::Statement(Statement::Comment(" End of implementation".to_string()))
                    .at_rc(4, 22),
            ]
        );
    }

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
                    DeclaredName::bare("A").at_rc(2, 22),
                    DeclaredName::bare("B").at_rc(2, 25)
                ],
                vec![Statement::Assignment(
                    "Add".into(),
                    Expression::BinaryExpression(
                        Operand::Plus,
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
                    DeclaredName::bare("a").at_rc(2, 22),
                    DeclaredName::bare("b").at_rc(2, 25)
                ],
                vec![Statement::Assignment(
                    "add".into(),
                    Expression::BinaryExpression(
                        Operand::Plus,
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
}
