use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::pc::copy::*;
use crate::parser::pc::loc::*;
use crate::parser::types::*;
use crate::variant;
use std::io::BufRead;

// TODO add test demand space after "AND" but not if parenthesis follows

pub fn expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        if_first_maybe_second_peeking_first(single_expression_node(), |reader, first_expr_ref| {
            if_first_demand_second(
                operand(first_expr_ref.is_parenthesis()),
                skipping_whitespace_lazy(expression_node),
                || QError::SyntaxError("Expected right side expression".to_string()),
            )(reader)
        }),
        |(l, r)| {
            match r {
                None => l,
                Some((Locatable { element: op, pos }, right_side)) => {
                    l.apply_priority_order(right_side, op, pos)
                }
            }
            .simplify_unary_minus_literals()
        },
    )
}

pub fn single_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    or_vec(vec![
        with_pos(string_literal::string_literal()),
        with_pos(word::word()),
        number_literal::number_literal(),
        number_literal::float_without_leading_zero(),
        with_pos(file_handle()),
        with_pos(parenthesis()),
        unary_not(),
        unary_minus(),
    ])
}

pub fn unary_minus<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        if_first_demand_second_lazy(with_pos(try_read('-')), expression_node, || {
            QError::SyntaxError("Expected expression after unary minus".to_string())
        }),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Minus, l.pos()),
    )
}

pub fn unary_not<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
    map(
        with_some_whitespace_between_lazy(
            with_pos(try_read_keyword(Keyword::Not)),
            expression_node,
            || QError::SyntaxError("Expected expression after NOT".to_string()),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Not, l.pos()),
    )
}

pub fn file_handle<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QError>)> {
    map_to_result_no_undo(
        if_first_demand_second(try_read('#'), with_pos(read_any_digits()), || {
            QError::SyntaxError("Expected digits after #".to_string())
        }),
        |(
            _,
            Locatable {
                element: digits, ..
            },
        )| match digits.parse::<u32>() {
            Ok(d) => Ok(Expression::FileHandle(d.into())),
            Err(err) => Err(err.into()),
        },
    )
}

pub fn parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QError>)> {
    // TODO allow skipping whitespace inside parenthesis
    map(in_parenthesis_lazy(expression_node), |v| {
        Expression::Parenthesis(Box::new(v))
    })
}

mod string_literal {
    use super::*;

    pub fn string_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QError>)> {
        map(
            if_first_demand_second(
                if_first_maybe_second(try_read('"'), read_any_str_while(|ch| ch != '"')),
                try_read('"'),
                || QError::SyntaxError("Unterminated string".to_string()),
            ),
            |((_, opt_str), _)| Expression::StringLiteral(opt_str.unwrap_or_default()),
        )
    }
}

mod number_literal {
    use super::*;

    pub fn number_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
        map_to_result_no_undo(
            if_first_maybe_second(
                with_pos(read_any_digits()),
                if_first_maybe_second(
                    if_first_demand_second(try_read('.'), read_any_digits(), || {
                        QError::SyntaxError("Expected digits after decimal point".to_string())
                    }),
                    try_read('#'),
                ),
            ),
            |(
                Locatable {
                    element: int_digits,
                    pos,
                },
                opt_frac,
            )| match opt_frac {
                Some(((_, frac_digits), opt_double)) => {
                    parse_floating_point_literal(int_digits, frac_digits, opt_double.is_some(), pos)
                }
                None => integer_literal_to_expression_node(int_digits, pos),
            },
        )
    }

    pub fn float_without_leading_zero<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QError>)> {
        map_to_result_no_undo(
            if_first_maybe_second(
                if_first_demand_second(with_pos(try_read('.')), read_any_digits(), || {
                    QError::SyntaxError("Expected digits after decimal point".to_string())
                }),
                try_read('#'),
            ),
            |((Locatable { pos, .. }, frac_digits), opt_double)| {
                parse_floating_point_literal(
                    "0".to_string(),
                    frac_digits,
                    opt_double.is_some(),
                    pos,
                )
            },
        )
    }

    fn integer_literal_to_expression_node(
        s: String,
        pos: Location,
    ) -> Result<ExpressionNode, QError> {
        match s.parse::<u32>() {
            Ok(u) => {
                if u <= variant::MAX_INTEGER as u32 {
                    Ok(Expression::IntegerLiteral(u as i32).at(pos))
                } else if u <= variant::MAX_LONG as u32 {
                    Ok(Expression::LongLiteral(u as i64).at(pos))
                } else {
                    Ok(Expression::DoubleLiteral(u as f64).at(pos))
                }
            }
            Err(e) => Err(e.into()),
        }
    }

    fn parse_floating_point_literal(
        integer_digits: String,
        fraction_digits: String,
        is_double: bool,
        pos: Location,
    ) -> Result<ExpressionNode, QError> {
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f).at(pos)),
                Err(err) => Err(err.into()),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f).at(pos)),
                Err(err) => Err(err.into()),
            }
        }
    }
}

mod word {
    use super::*;

    pub fn word<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QError>)> {
        map(
            if_first_maybe_second(
                name::name(),
                in_parenthesis(csv_one_or_more_lazy(expression_node, || {
                    QError::SyntaxError("Expected expression".to_string())
                })),
            ),
            |(n, opt_v)| match opt_v {
                Some(v) => Expression::FunctionCall(n, v),
                None => Expression::VariableName(n),
            },
        )
    }
}

pub fn operand<T: BufRead + 'static>(
    had_parenthesis_before: bool,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Locatable<Operand>, QError>)> {
    or_vec(vec![
        skipping_whitespace(with_pos(lte())),
        skipping_whitespace(with_pos(gte())),
        map(skipping_whitespace(with_pos(try_read('='))), |x| {
            x.map(|_| Operand::Equal)
        }),
        map(skipping_whitespace(with_pos(try_read('+'))), |x| {
            x.map(|_| Operand::Plus)
        }),
        map(skipping_whitespace(with_pos(try_read('-'))), |x| {
            x.map(|_| Operand::Minus)
        }),
        map(skipping_whitespace(with_pos(try_read('*'))), |x| {
            x.map(|_| Operand::Multiply)
        }),
        map(skipping_whitespace(with_pos(try_read('/'))), |x| {
            x.map(|_| Operand::Divide)
        }),
        if had_parenthesis_before {
            // skip whitespace + AND
            map(
                skipping_whitespace(with_pos(try_read_keyword(Keyword::And))),
                |x| x.map(|_| Operand::And),
            )
        } else {
            // demand whitespace + AND
            map(
                and(
                    read_any_whitespace(),
                    with_pos(try_read_keyword(Keyword::And)),
                ),
                |(_, x)| x.map(|_| Operand::And),
            )
        },
        if had_parenthesis_before {
            // skip whitespace + OR
            map(
                skipping_whitespace(with_pos(try_read_keyword(Keyword::Or))),
                |x| x.map(|_| Operand::Or),
            )
        } else {
            // demand whitespace + OR
            map(
                and(
                    read_any_whitespace(),
                    with_pos(try_read_keyword(Keyword::Or)),
                ),
                |(_, x)| x.map(|_| Operand::Or),
            )
        },
    ])
}

fn lte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Operand, QError>)> {
    map_to_result_no_undo(
        if_first_maybe_second(
            try_read('<'),
            with_pos(read_any_if(|ch| ch == '=' || ch == '>')),
        ),
        |(_, opt_r)| match opt_r {
            Some(Locatable { element: '=', .. }) => Ok(Operand::LessOrEqual),
            Some(Locatable { element: '>', .. }) => Ok(Operand::NotEqual),
            None => Ok(Operand::Less),
            Some(Locatable { element, .. }) => Err(QError::SyntaxError(format!(
                "Invalid character {} after <",
                element
            ))),
        },
    )
}

fn gte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Operand, QError>)> {
    map(
        if_first_maybe_second(try_read('>'), try_read('=')),
        |(_, opt_r)| match opt_r {
            Some(_) => Operand::GreaterOrEqual,
            _ => Operand::Greater,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Name, Operand, Statement, UnaryOperand};

    macro_rules! assert_expression {
        ($left:expr, $right:expr) => {
            let program = parse(format!("PRINT {}", $left)).demand_single_statement();
            match program {
                Statement::SubCall(_, args) => {
                    assert_eq!(1, args.len());
                    let first_arg_node = &args[0];
                    let Locatable {
                        element: first_arg, ..
                    } = first_arg_node;
                    assert_eq!(first_arg, &$right);
                }
                _ => panic!("Expected sub-call"),
            }
        };
    }

    macro_rules! assert_literal_expression {
        ($left:expr, $right:expr) => {
            assert_expression!($left, Expression::from($right));
        };
    }

    macro_rules! assert_variable_expression {
        ($left:expr, $right:expr) => {
            assert_expression!($left, Expression::VariableName(Name::from($right)));
        };
    }

    #[test]
    fn test_parse_literals() {
        assert_literal_expression!(r#""hello, world""#, "hello, world");
        assert_literal_expression!(r#""hello 123 . AS""#, "hello 123 . AS");
        assert_literal_expression!("42", 42);
        assert_literal_expression!("4.2", 4.2_f32);
        assert_literal_expression!("0.5", 0.5_f32);
        assert_literal_expression!(".5", 0.5_f32);
        assert_literal_expression!("3.14#", 3.14_f64);
        assert_literal_expression!("-42", -42);
    }

    #[test]
    fn test_variable_expression() {
        assert_variable_expression!("A", "A");
    }

    mod function_call {
        use super::*;

        #[test]
        fn test_function_call_expression_no_args() {
            assert_eq!(
                parse_err("PRINT IsValid()"),
                QError::SyntaxError("Expected expression".to_string())
            );
        }

        #[test]
        fn test_function_call_qualified_expression_no_args() {
            assert_eq!(
                parse_err("PRINT IsValid%()"),
                QError::SyntaxError("Expected expression".to_string())
            );
        }

        #[test]
        fn test_function_call_expression_one_arg() {
            assert_expression!(
                "IsValid(42)",
                Expression::FunctionCall(Name::from("IsValid"), vec![42.as_lit_expr(1, 15)])
            );
        }

        #[test]
        fn test_function_call_expression_two_args() {
            assert_expression!(
                "CheckProperty(42, \"age\")",
                Expression::FunctionCall(
                    Name::from("CheckProperty"),
                    vec![42.as_lit_expr(1, 21), "age".as_lit_expr(1, 25)]
                )
            );
        }

        #[test]
        fn test_function_call_in_function_call() {
            assert_expression!(
                "CheckProperty(LookupName(\"age\"), Confirm(1))",
                Expression::FunctionCall(
                    Name::from("CheckProperty"),
                    vec![
                        Expression::FunctionCall(
                            Name::from("LookupName"),
                            vec!["age".as_lit_expr(1, 32)]
                        )
                        .at_rc(1, 21),
                        Expression::FunctionCall(Name::from("Confirm"), vec![1.as_lit_expr(1, 48)])
                            .at_rc(1, 40)
                    ]
                )
            );
        }
    }

    #[test]
    fn test_lte() {
        assert_expression!(
            "N <= 1",
            Expression::BinaryExpression(
                Operand::LessOrEqual,
                Box::new("N".as_var_expr(1, 7)),
                Box::new(1.as_lit_expr(1, 12)),
            )
        );
    }

    #[test]
    fn test_less_than() {
        assert_expression!(
            "A < B",
            Expression::BinaryExpression(
                Operand::Less,
                Box::new("A".as_var_expr(1, 7)),
                Box::new("B".as_var_expr(1, 11)),
            )
        );
    }

    mod priority {
        use super::*;

        #[test]
        fn test_a_plus_b_less_than_c() {
            assert_expression!(
                "A + B < C",
                Expression::BinaryExpression(
                    Operand::Less,
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new("A".as_var_expr(1, 7)),
                            Box::new("B".as_var_expr(1, 11))
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new("C".as_var_expr(1, 15))
                )
            );
        }

        #[test]
        fn test_a_plus_parenthesis_b_less_than_c() {
            assert_expression!(
                "A + (B < C)",
                Expression::BinaryExpression(
                    Operand::Plus,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(
                        Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operand::Less,
                                Box::new("B".as_var_expr(1, 12)),
                                Box::new("C".as_var_expr(1, 16))
                            )
                            .at_rc(1, 14)
                        ))
                        .at_rc(1, 11)
                    )
                )
            );
        }

        #[test]
        fn test_a_less_than_b_plus_c() {
            assert_expression!(
                "A < B + C",
                Expression::BinaryExpression(
                    Operand::Less,
                    Box::new("A".as_var_expr(1, 7)),
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new("B".as_var_expr(1, 11)),
                            Box::new("C".as_var_expr(1, 15))
                        )
                        .at_rc(1, 13)
                    )
                )
            );
        }

        #[test]
        fn test_parenthesis_a_less_than_b_end_parenthesis_plus_c() {
            assert_expression!(
                "(A < B) + C",
                Expression::BinaryExpression(
                    Operand::Plus,
                    Box::new(
                        Expression::Parenthesis(Box::new(
                            Expression::BinaryExpression(
                                Operand::Less,
                                Box::new("A".as_var_expr(1, 8)),
                                Box::new("B".as_var_expr(1, 12))
                            )
                            .at_rc(1, 10)
                        ))
                        .at_rc(1, 7)
                    ),
                    Box::new("C".as_var_expr(1, 17)),
                )
            );
        }

        #[test]
        fn test_a_gt_0_and_b_lt_1() {
            assert_expression!(
                "A > 0 AND B < 1",
                Expression::BinaryExpression(
                    Operand::And,
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Greater,
                            Box::new("A".as_var_expr(1, 7)),
                            Box::new(0.as_lit_expr(1, 11)),
                        )
                        .at_rc(1, 9)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Less,
                            Box::new("B".as_var_expr(1, 17)),
                            Box::new(1.as_lit_expr(1, 21)),
                        )
                        .at_rc(1, 19)
                    )
                )
            );
        }

        #[test]
        fn test_not_eof_1_and_id_gt_0() {
            assert_expression!(
                "NOT EOF(1) AND ID > 0",
                Expression::BinaryExpression(
                    Operand::And,
                    Box::new(
                        Expression::UnaryExpression(
                            UnaryOperand::Not,
                            Box::new(
                                Expression::FunctionCall(
                                    Name::from("EOF"),
                                    vec![1.as_lit_expr(1, 15)]
                                )
                                .at_rc(1, 11)
                            )
                        )
                        .at_rc(1, 7)
                    ),
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Greater,
                            Box::new("ID".as_var_expr(1, 22)),
                            Box::new(0.as_lit_expr(1, 27))
                        )
                        .at_rc(1, 25)
                    )
                )
            );
        }

        #[test]
        fn test_negated_number_and_positive_number() {
            assert_expression!(
                "-5 AND 2",
                Expression::BinaryExpression(
                    Operand::And,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 14))
                )
            );
        }

        #[test]
        fn test_negated_number_plus_positive_number() {
            assert_expression!(
                "-5 + 2",
                Expression::BinaryExpression(
                    Operand::Plus,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 12))
                )
            );
        }

        #[test]
        fn test_negated_number_lt_positive_number() {
            assert_expression!(
                "-5 < 2",
                Expression::BinaryExpression(
                    Operand::Less,
                    Box::new((-5_i32).as_lit_expr(1, 7)),
                    Box::new(2.as_lit_expr(1, 12))
                )
            );
        }
    }

    mod binary_plus {
        use super::*;

        #[test]
        fn test_plus() {
            assert_expression!(
                "N + 1",
                Expression::BinaryExpression(
                    Operand::Plus,
                    Box::new("N".as_var_expr(1, 7)),
                    Box::new(1.as_lit_expr(1, 11)),
                )
            );
        }

        #[test]
        fn test_plus_three() {
            assert_expression!(
                "N + 1 + 2",
                Expression::BinaryExpression(
                    Operand::Plus,
                    Box::new("N".as_var_expr(1, 7)),
                    Box::new(
                        Expression::BinaryExpression(
                            Operand::Plus,
                            Box::new(1.as_lit_expr(1, 11)),
                            Box::new(2.as_lit_expr(1, 15))
                        )
                        .at_rc(1, 13)
                    )
                )
            );
        }
    }

    #[test]
    fn test_minus() {
        assert_expression!(
            "N - 2",
            Expression::BinaryExpression(
                Operand::Minus,
                Box::new("N".as_var_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 11)),
            )
        );
    }

    #[test]
    fn test_negated_variable() {
        assert_expression!(
            "-N",
            Expression::UnaryExpression(UnaryOperand::Minus, Box::new("N".as_var_expr(1, 8)))
        );
    }

    #[test]
    fn test_negated_number_literal_resolved_eagerly_during_parsing() {
        assert_expression!("-42", Expression::IntegerLiteral(-42));
    }

    #[test]
    fn test_fib_expression() {
        assert_expression!(
            "Fib(N - 1) + Fib(N - 2)",
            Expression::BinaryExpression(
                Operand::Plus,
                Box::new(
                    Expression::FunctionCall(
                        Name::from("Fib"),
                        vec![Expression::BinaryExpression(
                            Operand::Minus,
                            Box::new("N".as_var_expr(1, 11)),
                            Box::new(1.as_lit_expr(1, 15)),
                        )
                        .at_rc(1, 13)],
                    )
                    .at_rc(1, 7)
                ),
                Box::new(
                    Expression::FunctionCall(
                        Name::from("Fib"),
                        vec![Expression::BinaryExpression(
                            Operand::Minus,
                            Box::new("N".as_var_expr(1, 24)),
                            Box::new(2.as_lit_expr(1, 28)),
                        )
                        .at_rc(1, 26)],
                    )
                    .at_rc(1, 20)
                ),
            )
        );
    }

    #[test]
    fn test_negated_function_call() {
        assert_expression!(
            "-Fib(-N)",
            Expression::UnaryExpression(
                UnaryOperand::Minus,
                Box::new(
                    Expression::FunctionCall(
                        Name::from("Fib"),
                        vec![Expression::UnaryExpression(
                            UnaryOperand::Minus,
                            Box::new("N".as_var_expr(1, 13)),
                        )
                        .at_rc(1, 12)],
                    )
                    .at_rc(1, 8)
                )
            )
        );
    }

    #[test]
    fn test_and_or_leading_whitespace() {
        assert_expression!(
            "1 AND 2",
            Expression::BinaryExpression(
                Operand::And,
                Box::new(1.as_lit_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 13))
            )
        );
        assert_eq!(
            parse_err("PRINT 1AND 2"),
            QError::SyntaxError("No separator: A".to_string())
        );
        assert_expression!(
            "(1 OR 2)AND 3",
            Expression::BinaryExpression(
                Operand::And,
                Box::new(
                    Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operand::Or,
                            Box::new(1.as_lit_expr(1, 8)),
                            Box::new(2.as_lit_expr(1, 13))
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19))
            )
        );
        assert_expression!(
            "1 OR 2",
            Expression::BinaryExpression(
                Operand::Or,
                Box::new(1.as_lit_expr(1, 7)),
                Box::new(2.as_lit_expr(1, 12))
            )
        );
        assert_eq!(
            parse_err("PRINT 1OR 2"),
            QError::SyntaxError("No separator: O".to_string())
        );
        assert_expression!(
            "(1 AND 2)OR 3",
            Expression::BinaryExpression(
                Operand::Or,
                Box::new(
                    Expression::Parenthesis(Box::new(
                        Expression::BinaryExpression(
                            Operand::And,
                            Box::new(1.as_lit_expr(1, 8)),
                            Box::new(2.as_lit_expr(1, 14))
                        )
                        .at_rc(1, 10)
                    ))
                    .at_rc(1, 7)
                ),
                Box::new(3.as_lit_expr(1, 19))
            )
        );
    }
}
