use crate::char_reader::*;
use crate::common::*;
use crate::lexer::BufLexer;
use crate::lexer::Keyword;
use crate::parser::def_type;
use crate::parser::top_level_token;
use crate::parser::types::*;
use std::fs::File;
use std::io::BufRead;

pub fn parse_main_file(f: File) -> Result<ProgramNode, QErrorNode> {
    let mut lexer = BufLexer::from(f);
    parse_main(&mut lexer)
}

#[cfg(test)]
pub fn parse_main_str<T: AsRef<[u8]> + 'static>(s: T) -> Result<ProgramNode, QErrorNode> {
    parse_main_str_old(s)
}

fn parse_main_str_old<T: AsRef<[u8]> + 'static>(s: T) -> Result<ProgramNode, QErrorNode> {
    let mut lexer = BufLexer::from(s);
    parse_main(&mut lexer)
}

pub fn parse_main<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<ProgramNode, QErrorNode> {
    top_level_token::parse_top_level_tokens(lexer)
}

fn parse_main_str_new<T: AsRef<[u8]> + 'static>(s: T) -> Result<ProgramNode, QErrorNode> {
    let reader = EolReader::from(s);
    let (_, result) = top_level_tokens()(reader);
    // TODO verify reader does not have any more characters left, i.e. it was fully parsed
    result
}

pub fn top_level_tokens<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ProgramNode, QErrorNode>)> {
    Box::new(move |r| {
        let mut read_separator = true; // we are at the beginning of the file
        let mut top_level_tokens: ProgramNode = vec![];
        let mut reader = r;
        loop {
            let (tmp, next) = reader.read();
            reader = tmp;
            match next {
                Ok(' ') => {
                    // skip whitespace
                }
                Ok('\r') | Ok('\n') | Ok(':') => {
                    read_separator = true;
                }
                Err(err) => {
                    if err.is_not_found_err() {
                        break;
                    } else {
                        return (reader, Err(err));
                    }
                }
                Ok(ch) => {
                    // if it is a comment, we are allowed to read it without a separator
                    let can_read = ch == '\'' || read_separator;
                    if can_read {
                        let (tmp, next) = top_level_token_one()(reader.undo(ch));
                        reader = tmp;
                        read_separator = false;
                        match next {
                            Ok(top_level_token) => {
                                top_level_tokens.push(top_level_token);
                            }
                            Err(err) => {
                                if err.is_not_found_err() {
                                    return (
                                        reader,
                                        Err(err.map(|_| {
                                            QError::SyntaxError(format!(
                                                "Expected top level statement"
                                            ))
                                        })),
                                    );
                                } else {
                                    return (reader, Err(err));
                                }
                            }
                        }
                    } else {
                        return reader.err(QError::SyntaxError(format!("No separator")));
                    }
                }
            }
        }

        (reader, Ok(top_level_tokens))
    })
}

pub fn top_level_token_one<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelTokenNode, QErrorNode>)> {
    with_pos(or_vec(vec![
        top_level_token_def_type(),
        top_level_token_declaration(),
        top_level_token_implementation(),
        top_level_token_statement(),
    ]))
}

pub fn top_level_token_def_type<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    apply(def_type::take_def_type(), |d| TopLevelToken::DefType(d))
}

pub fn top_level_token_declaration<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn top_level_token_implementation<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn top_level_token_statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<TopLevelToken, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statements<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNodes, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_dim<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_const<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_comment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_built_ins<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_sub_call<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_assignment<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_label<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_if_block<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_for_loop<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_select_case<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_while_wend<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_on_error_go_to<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    Box::new(move |reader| (reader, Err(QErrorNode::NoPos(QError::FeatureUnavailable))))
}

pub fn statement_illegal_keywords<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<StatementNode, QErrorNode>)> {
    or(
        switch(with_pos(try_read_keyword(Keyword::Wend)), |k| {
            Err(QError::WendWithoutWhile).with_err_at(k)
        }),
        switch(with_pos(try_read_keyword(Keyword::Else)), |k| {
            Err(QError::ElseWithoutIf).with_err_at(k)
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::types::*;

    #[test]
    fn test_parse_fixture_fib() {
        let program = parse_file("FIB.BAS").strip_location();
        assert_eq!(
            program,
            vec![
                // DECLARE FUNCTION Fib! (N!)
                TopLevelToken::FunctionDeclaration(
                    "Fib!".as_name(1, 18),
                    vec![DeclaredName::compact("N", TypeQualifier::BangSingle).at_rc(1, 24)],
                ),
                // PRINT "Enter the number of fibonacci to calculate"
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("PRINT"),
                    vec!["Enter the number of fibonacci to calculate".as_lit_expr(2, 7)],
                )),
                // INPUT N
                TopLevelToken::Statement(Statement::SubCall(
                    BareName::from("INPUT"),
                    vec!["N".as_var_expr(3, 7)]
                )),
                // FOR I = 0 TO N
                TopLevelToken::Statement(Statement::ForLoop(ForLoopNode {
                    variable_name: "I".as_name(4, 5),
                    lower_bound: 0.as_lit_expr(4, 9),
                    upper_bound: "N".as_var_expr(4, 14),
                    step: None,
                    statements: vec![
                        // PRINT "Fibonacci of ", I, " is ", Fib(I)
                        Statement::SubCall(
                            BareName::from("PRINT"),
                            vec![
                                "Fibonacci of".as_lit_expr(5, 11),
                                "I".as_var_expr(5, 27),
                                "is".as_lit_expr(5, 30),
                                Expression::FunctionCall(
                                    Name::from("Fib"),
                                    vec!["I".as_var_expr(5, 40)],
                                )
                                .at_rc(5, 36),
                            ],
                        )
                        .at_rc(5, 5),
                    ],
                    next_counter: None,
                })),
                // FUNCTION Fib (N)
                TopLevelToken::FunctionImplementation(
                    Name::from("Fib").at_rc(8, 10),
                    vec![DeclaredName::bare("N").at_rc(8, 15)],
                    vec![
                        // IF N <= 1 THEN
                        Statement::IfBlock(IfBlockNode {
                            if_block: ConditionalBlockNode {
                                // N <= 1
                                condition: Expression::BinaryExpression(
                                    Operand::LessOrEqual,
                                    Box::new("N".as_var_expr(9, 8)),
                                    Box::new(1.as_lit_expr(9, 13))
                                )
                                .at_rc(9, 10),
                                statements: vec![
                                    // Fib = N
                                    Statement::Assignment(
                                        Name::from("Fib"),
                                        "N".as_var_expr(10, 15)
                                    )
                                    .at_rc(10, 9)
                                ],
                            },
                            else_if_blocks: vec![],
                            else_block: Some(vec![
                                // ELSE Fib = Fib(N - 1) + Fib(N - 2)
                                Statement::Assignment(
                                    Name::from("Fib"),
                                    Expression::BinaryExpression(
                                        Operand::Plus,
                                        Box::new(
                                            Expression::FunctionCall(
                                                Name::from("Fib"),
                                                vec![Expression::BinaryExpression(
                                                    Operand::Minus,
                                                    Box::new("N".as_var_expr(12, 19)),
                                                    Box::new(1.as_lit_expr(12, 23)),
                                                )
                                                .at_rc(12, 21)]
                                            )
                                            .at_rc(12, 15)
                                        ),
                                        Box::new(
                                            Expression::FunctionCall(
                                                Name::from("Fib"),
                                                vec![Expression::BinaryExpression(
                                                    Operand::Minus,
                                                    Box::new("N".as_var_expr(12, 32)),
                                                    Box::new(2.as_lit_expr(12, 36)),
                                                )
                                                .at_rc(12, 34)]
                                            )
                                            .at_rc(12, 28)
                                        )
                                    )
                                    .at_rc(12, 26)
                                )
                                .at_rc(12, 9)
                            ])
                        })
                        .at_rc(9, 5)
                    ],
                ),
            ],
        );
    }
}
