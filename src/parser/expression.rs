use crate::common::*;
use crate::parser::char_reader::*;
use crate::parser::name;
use crate::parser::pc::common::*;
use crate::parser::pc::copy::*;
use crate::parser::pc::loc::*;
use crate::parser::pc::map::{and_then, map};
use crate::parser::pc::*;
use crate::parser::types::*;
use crate::variant;
use std::io::BufRead;

pub fn demand_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    demand(
        expression_node(),
        QError::syntax_error_fn("Expected: expression"),
    )
}

fn guarded_parenthesis_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    map(
        seq3(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('('))),
            // caveat: once it reads the opening parenthesis, it goes into demand mode,
            // which is not consistent with the name of the function
            demand_expression_node(),
            demand(try_read(')'), QError::syntax_error_fn("Expected: )")),
        ),
        |(Locatable { pos, .. }, e, _)| Expression::Parenthesis(Box::new(e)).at(pos),
    )
}

fn guarded_whitespace_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws+ expr
    crate::parser::pc::ws::one_or_more_leading(expression_node())
}

pub fn guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    // ws+ expr
    or(
        guarded_parenthesis_expression_node(),
        guarded_whitespace_expression_node(),
    )
}

pub fn demand_guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    // ws+ expr
    demand(
        guarded_expression_node(),
        QError::syntax_error_fn("Expected: expression"),
    )
}

pub fn demand_back_guarded_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // ws* ( expr )
    // ws+ expr ws+
    demand(
        or(
            guarded_parenthesis_expression_node(),
            drop_right(seq2(
                guarded_whitespace_expression_node(),
                demand(
                    crate::parser::pc::ws::one_or_more(),
                    QError::syntax_error_fn("Expected: whitespace after expression"),
                ),
            )),
        ),
        QError::syntax_error_fn("Expected: expression"),
    )
}

/// Parses an expression
pub fn expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    // TODO just use source_and_then_some
    Box::new(move |reader| match single_expression_node()(reader) {
        Ok((reader, opt_res)) => match opt_res {
            Some(first_expr) => {
                try_second_expression(reader, first_expr).and_then(|(reader, res)| {
                    Ok((reader, res.map(|x| x.simplify_unary_minus_literals())))
                })
            }
            None => Ok((reader, None)),
        },
        Err(err) => Err(err),
    })
}

// TODO refactor this
fn try_second_expression<T: BufRead + 'static>(
    reader: EolReader<T>,
    first_expr: ExpressionNode,
) -> ReaderResult<EolReader<T>, ExpressionNode, QError> {
    match seq2(
        operand(first_expr.is_parenthesis()),
        demand(
            crate::parser::pc::ws::zero_or_more_leading(lazy(expression_node)),
            QError::syntax_error_fn("Expected: right side expression"),
        ),
    )(reader)
    {
        Ok((reader, opt_second)) => match opt_second {
            Some((loc_op, second_expr)) => {
                let Locatable { element: op, pos } = loc_op;
                Ok((
                    reader,
                    Some(first_expr.apply_priority_order(second_expr, op, pos)),
                ))
            }
            None => Ok((reader, Some(first_expr))),
        },
        Err(err) => Err(err),
    }
}

fn single_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        seq2(
            with_pos(try_read('-')),
            demand(
                lazy(expression_node),
                QError::syntax_error_fn("Expected: expression after unary minus"),
            ),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Minus, l.pos()),
    )
}

pub fn unary_not<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
    map(
        crate::parser::pc::ws::seq2(
            with_pos(try_read_keyword(Keyword::Not)),
            demand(
                lazy(expression_node),
                QError::syntax_error_fn("Expected: expression after NOT"),
            ),
            QError::syntax_error_fn("Expected: whitespace after NOT"),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Not, l.pos()),
    )
}

pub fn file_handle<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
    and_then(
        seq2(
            try_read('#'),
            demand(
                read_any_digits(),
                QError::syntax_error_fn("Expected: digits after #"),
            ),
        ),
        |(_, digits)| match digits.parse::<u32>() {
            Ok(d) => Ok(Expression::FileHandle(d.into())),
            Err(err) => Err(err.into()),
        },
    )
}

pub fn parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
    map(
        in_parenthesis(crate::parser::pc::ws::zero_or_more_around(lazy(
            expression_node,
        ))),
        |v| Expression::Parenthesis(Box::new(v)),
    )
}

mod string_literal {
    use super::*;

    fn is_not_quote(ch: char) -> bool {
        ch != '"'
    }

    pub fn string_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
        map(
            seq3(
                try_read('"'),
                crate::parser::pc::str::zero_or_more_if(is_not_quote),
                demand(
                    try_read('"'),
                    QError::syntax_error_fn("Unterminated string"),
                ),
            ),
            |(_, s, _)| Expression::StringLiteral(s),
        )
    }
}

mod number_literal {
    use super::*;

    pub fn number_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        and_then(
            opt_seq3(
                with_pos(read_any_digits()),
                seq2(
                    with_pos(try_read('.')),
                    demand(
                        read_any_digits(),
                        QError::syntax_error_fn("Expected: digits after decimal point"),
                    ),
                ),
                try_read('#'),
            ),
            |(
                Locatable {
                    element: int_digits,
                    pos,
                },
                opt_frac,
                opt_double,
            )| match opt_frac {
                Some((_, frac_digits)) => {
                    parse_floating_point_literal(int_digits, frac_digits, opt_double.is_some(), pos)
                }
                None => integer_literal_to_expression_node(int_digits, pos),
            },
        )
    }

    pub fn float_without_leading_zero<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, ExpressionNode, QError>> {
        and_then(
            opt_seq3(
                with_pos(try_read('.')),
                demand(
                    read_any_digits(),
                    QError::syntax_error_fn("Expected: digits after decimal point"),
                ),
                try_read('#'),
            ),
            |(Locatable { pos, .. }, opt_frac_digits, opt_double)| {
                parse_floating_point_literal(
                    "0".to_string(),
                    opt_frac_digits.unwrap(),
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
    ) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Expression, QError>> {
        map(
            opt_seq2(
                name::name(),
                in_parenthesis(and_then(csv_zero_or_more(lazy(expression_node)), |v| {
                    if v.is_empty() {
                        Err(QError::syntax_error(
                            "Cannot have function call without arguments",
                        ))
                    } else {
                        Ok(v)
                    }
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Locatable<Operand>, QError>> {
    or_vec(vec![
        crate::parser::pc::ws::zero_or_more_leading(with_pos(lte())),
        crate::parser::pc::ws::zero_or_more_leading(with_pos(gte())),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('='))),
            |x| x.map(|_| Operand::Equal),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('+'))),
            |x| x.map(|_| Operand::Plus),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('-'))),
            |x| x.map(|_| Operand::Minus),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('*'))),
            |x| x.map(|_| Operand::Multiply),
        ),
        map(
            crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read('/'))),
            |x| x.map(|_| Operand::Divide),
        ),
        if had_parenthesis_before {
            // skip whitespace + AND
            map(
                crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read_keyword(
                    Keyword::And,
                ))),
                |x| x.map(|_| Operand::And),
            )
        } else {
            // demand whitespace + AND
            map(
                crate::parser::pc::ws::one_or_more_leading(with_pos(try_read_keyword(
                    Keyword::And,
                ))),
                |locatable| locatable.map(|_| Operand::And),
            )
        },
        if had_parenthesis_before {
            // skip whitespace + OR
            map(
                crate::parser::pc::ws::zero_or_more_leading(with_pos(try_read_keyword(
                    Keyword::Or,
                ))),
                |x| x.map(|_| Operand::Or),
            )
        } else {
            // demand whitespace + OR
            map(
                crate::parser::pc::ws::one_or_more_leading(with_pos(try_read_keyword(Keyword::Or))),
                |locatable| locatable.map(|_| Operand::Or),
            )
        },
    ])
}

fn lte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Operand, QError>> {
    and_then(
        opt_seq2(
            try_read('<'),
            with_pos(read_if(|ch| ch == '=' || ch == '>')),
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
) -> Box<dyn Fn(EolReader<T>) -> ReaderResult<EolReader<T>, Operand, QError>> {
    map(
        opt_seq2(try_read('>'), try_read('=')),
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
                _ => panic!("Expected: sub-call"),
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
                QError::SyntaxError("Cannot have function call without arguments".to_string())
            );
        }

        #[test]
        fn test_function_call_qualified_expression_no_args() {
            assert_eq!(
                parse_err("PRINT IsValid%()"),
                QError::SyntaxError("Cannot have function call without arguments".to_string())
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
