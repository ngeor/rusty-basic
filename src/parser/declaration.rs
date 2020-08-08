use crate::common::*;
use crate::lexer::{BufLexer, Keyword, LexemeNode};
use crate::parser::buf_lexer::*;
use crate::parser::declared_name;
use crate::parser::error::*;
use crate::parser::name;
use crate::parser::types::*;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Declare) {
        return Ok(None);
    }

    let pos = lexer.read()?.pos();
    read_demand_whitespace(lexer, "Expected whitespace after DECLARE keyword")?;
    let next = lexer.read()?;
    match next {
        LexemeNode::Keyword(Keyword::Function, _, _) => {
            read_demand_whitespace(lexer, "Expected whitespace after FUNCTION keyword")?;
            let function_name = demand(lexer, name::try_read, "Expected function name")?;
            let parameters = demand(
                lexer,
                try_read_declaration_parameters,
                "Expected function declaration parameters",
            )?;
            Ok(Some(
                TopLevelToken::FunctionDeclaration(function_name, parameters).at(pos),
            ))
        }
        LexemeNode::Keyword(Keyword::Sub, _, _) => {
            read_demand_whitespace(lexer, "Expected whitespace after SUB keyword")?;
            let sub_name = demand(lexer, name::try_read_bare, "Expected sub name")?;
            let parameters = demand(
                lexer,
                try_read_declaration_parameters,
                "Expected sub declaration parameters",
            )?;
            Ok(Some(
                TopLevelToken::SubDeclaration(sub_name, parameters).at(pos),
            ))
        }
        _ => unexpected("Unknown declaration", next),
    }
}

pub fn try_read_declaration_parameters<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<DeclaredNameNodes>, ParserError> {
    lexer.begin_transaction();
    skip_whitespace(lexer)?;
    match lexer.peek()? {
        LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => {
            // EOL: no parameters e.g. Sub Hello
            lexer.commit_transaction()?;
            Ok(Some(vec![]))
        }
        LexemeNode::Symbol('(', _) => {
            lexer.commit_transaction()?;
            parse_parameters(lexer).map(|x| Some(x))
        }
        _ => {
            lexer.rollback_transaction()?;
            Ok(None)
        }
    }
}

fn parse_parameters<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<DeclaredNameNodes, ParserError> {
    lexer.read()?; // read opening parenthesis
    skip_whitespace(lexer)?;
    match lexer.peek()? {
        LexemeNode::Word(_, _) => {
            // probably variable name
            let first_param = parse_one_parameter(lexer)?;
            let mut remaining = parse_next_parameter(lexer)?;
            let mut result: DeclaredNameNodes = vec![first_param];
            result.append(&mut remaining);
            Ok(result)
        }
        LexemeNode::Symbol(')', _) => {
            // exit e.g. Sub Hello()
            lexer.read()?;
            Ok(vec![])
        }
        _ => Err(ParserError::SyntaxError(
            "Expected parameter name or closing parenthesis".to_string(),
            lexer.peek()?.pos(),
        )),
    }
}

fn parse_next_parameter<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<DeclaredNameNodes, ParserError> {
    skip_whitespace(lexer)?;
    match lexer.peek()? {
        LexemeNode::Symbol(',', _) => {
            lexer.read()?;
            skip_whitespace(lexer)?;
            let first_param = parse_one_parameter(lexer)?;
            let mut remaining = parse_next_parameter(lexer)?;
            let mut result: DeclaredNameNodes = vec![first_param];
            result.append(&mut remaining);
            Ok(result)
        }
        LexemeNode::Symbol(')', _) => {
            lexer.read()?;
            Ok(vec![])
        }
        _ => Err(ParserError::SyntaxError(
            "Expected comma or closing parenthesis".to_string(),
            lexer.peek()?.pos(),
        )),
    }
}

fn parse_one_parameter<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<DeclaredNameNode, ParserError> {
    demand(lexer, declared_name::try_read, "Expected parameter")
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
