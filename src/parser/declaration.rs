use super::{unexpected, NameNode, ParserError, TopLevelToken, TopLevelTokenNode};
use crate::common::*;
use crate::lexer::{BufLexer, Keyword, LexemeNode};
use crate::parser::buf_lexer::*;
use crate::parser::name;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<TopLevelTokenNode>, ParserError> {
    if !lexer.peek()?.is_keyword(Keyword::Declare) {
        return Ok(None);
    }

    let pos = lexer.read()?.location();
    read_demand_whitespace(lexer, "Expected whitespace after DECLARE keyword")?;
    let next = lexer.read()?;
    match next {
        LexemeNode::Keyword(Keyword::Function, _, _) => {
            read_demand_whitespace(lexer, "Expected whitespace after FUNCTION keyword")?;
            let function_name = demand(lexer, name::try_read, "Expected function name")?;
            let parameters = parse_declaration_parameters(lexer)?;
            Ok(Some(
                TopLevelToken::FunctionDeclaration(function_name, parameters).at(pos),
            ))
        }
        LexemeNode::Keyword(Keyword::Sub, _, _) => {
            read_demand_whitespace(lexer, "Expected whitespace after SUB keyword")?;
            let sub_name = demand(lexer, name::try_read_bare, "Expected sub name")?;
            let parameters = parse_declaration_parameters(lexer)?;
            Ok(Some(
                TopLevelToken::SubDeclaration(sub_name, parameters).at(pos),
            ))
        }
        _ => unexpected("Unknown declaration", next),
    }
}

pub fn parse_declaration_parameters<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Vec<NameNode>, ParserError> {
    let mut params: Vec<NameNode> = vec![];
    skip_whitespace(lexer)?;
    let next = lexer.peek()?;
    if next.is_symbol('(') {
        lexer.read()?;
        parse_inside_parentheses(lexer, &mut params)?;
        Ok(params)
    } else if next.is_eol_or_eof() {
        // no parentheses e.g. DECLARE FUNCTION hello
        Ok(params)
    } else {
        unexpected("Expected ( or EOL or EOF after function name", next)
    }
}

fn parse_inside_parentheses<T: BufRead>(
    lexer: &mut BufLexer<T>,
    params: &mut Vec<NameNode>,
) -> Result<(), ParserError> {
    // holds the previous token, which can be one of:
    // '(' -> opening parenthesis (the starting point)
    // 'p' -> parameter
    // ',' -> comma
    let mut prev = '(';
    let mut found_close_parenthesis = false;
    while !found_close_parenthesis {
        skip_whitespace(lexer)?;
        let next = lexer.peek()?;
        match next {
            LexemeNode::Symbol(')', _) => {
                lexer.read()?;
                if prev == ',' {
                    return unexpected("Expected parameter after comma", next);
                } else {
                    found_close_parenthesis = true;
                }
            }
            LexemeNode::Symbol(',', _) => {
                lexer.read()?;
                if prev == 'p' {
                    prev = ',';
                } else {
                    return unexpected("Unexpected comma", next);
                }
            }
            LexemeNode::Word(_, _) => {
                if prev == '(' || prev == ',' {
                    params.push(demand(lexer, name::try_read, "Expected parameter")?);
                    prev = 'p';
                } else {
                    return unexpected("Unexpected name", next);
                }
            }
            _ => {
                return unexpected("Syntax error", next);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Operand, Statement, TopLevelToken};

    macro_rules! assert_function_declaration {
        ($input:expr, $expected_function_name:expr, $expected_params:expr) => {
            match parse($input).demand_single().as_ref() {
                TopLevelToken::FunctionDeclaration(name, parameters) => {
                    assert_eq!(name, $expected_function_name, "Function name mismatch");
                    let x = $expected_params;
                    assert_eq!(parameters.len(), x.len(), "Parameter count mismatch");
                    for i in 0..x.len() {
                        assert_eq!(&parameters[i], x[i], "Parameter {}", i);
                    }
                }
                _ => panic!(format!("{:?}", $input)),
            }
        };
    }

    #[test]
    fn test_fn() {
        assert_function_declaration!("DECLARE FUNCTION Fib! (N!)", "Fib!", vec!["N!"]);
    }

    #[test]
    fn test_lower_case() {
        assert_function_declaration!("declare function echo$(msg$)", "echo$", vec!["msg$"]);
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
                TopLevelToken::FunctionDeclaration("Echo".as_name(2, 26), vec!["X".as_name(2, 31)])
                    .at_rc(2, 9),
                TopLevelToken::Statement(Statement::Comment(" Echoes stuff back".to_string()))
                    .at_rc(2, 34),
                TopLevelToken::FunctionImplementation(
                    "Echo".as_name(3, 18),
                    vec!["X".as_name(3, 23)],
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
                vec!["A".as_name(2, 22), "B".as_name(2, 25)],
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
                vec!["a".as_name(2, 22), "b".as_name(2, 25)],
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
