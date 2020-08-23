use crate::char_reader::*;
use crate::common::pc::*;
use crate::common::*;
use crate::lexer::*;
use crate::parser::buf_lexer_helpers::*;
use crate::parser::name;
use crate::parser::types::*;
use crate::variant;
use std::io::BufRead;

// TODO add test demand space after "AND" but not if parenthesis follows

pub fn expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
    map_ng(
        if_first_maybe_second_peeking_first(single_expression_node(), |reader, first_expr_ref| {
            if_first_demand_second(
                operand(first_expr_ref.is_parenthesis()),
                expression_node(),
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

#[deprecated]
pub fn take_if_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>> {
    // boxed needed because otherwise rust complains about an infinite recursion on the
    // concrete type
    Box::new(move |input| {
        take_if_single_expression()(input).and_then_ok(|first_expression| {
            let opt_op = take_if_operand(&first_expression)(input);
            match opt_op {
                None => Some(Ok(first_expression.simplify_unary_minus_literals())),
                Some(Err(err)) => Some(Err(err)),
                Some(Ok(Locatable { element: op, pos })) => demand(
                    "Expected right side expression",
                    skipping_whitespace(take_if_expression_node()),
                )(input)
                .map_ok(|right_expr| {
                    first_expression
                        .apply_priority_order(right_expr, op, pos)
                        .simplify_unary_minus_literals()
                }),
            }
        })
    })
}

pub fn single_expression_node<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
    or_vec_ng(vec![
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

#[deprecated]
fn take_if_single_expression<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>> {
    or_vec(vec![
        string_literal::take_if_string_literal(),
        Box::new(word::take_if_word()),
        Box::new(number_literal::take_if_number_literal()),
        Box::new(number_literal::take_if_float_without_leading_zero()),
        Box::new(take_if_file_handle()),
        Box::new(take_if_parenthesis()),
        take_if_unary_not(),
        take_if_unary_minus(),
    ])
}

#[deprecated]
pub fn try_read<T: BufRead + 'static>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<ExpressionNode>, QErrorNode> {
    take_if_expression_node()(lexer).transpose()
}

pub fn unary_minus<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
    map_ng(
        if_first_demand_second(with_pos(try_read_char('-')), expression_node(), || {
            QError::SyntaxError("Expected expression after unary minus".to_string())
        }),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Minus, l.pos()),
    )
}

#[deprecated]
fn take_if_unary_minus<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>> {
    apply(
        |(l, child)| child.apply_unary_priority_order(UnaryOperand::Minus, l.pos()),
        and(
            take_if_symbol('-'),
            demand(
                "Expected expression after unary minus",
                take_if_expression_node(),
            ),
        ),
    )
}

pub fn unary_not<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
    map_ng(
        with_some_whitespace_between(
            with_pos(try_read_keyword(Keyword::Not)),
            expression_node(),
            || QError::SyntaxError("Expected expression after NOT".to_string()),
        ),
        |(l, r)| r.apply_unary_priority_order(UnaryOperand::Not, l.pos()),
    )
}

#[deprecated]
fn take_if_unary_not<T: BufRead + 'static>(
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>> {
    apply(
        |(l, child)| child.apply_unary_priority_order(UnaryOperand::Not, l.pos()),
        with_whitespace_between(take_if_keyword(Keyword::Not), take_if_expression_node()),
    )
}

pub fn file_handle<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QErrorNode>)> {
    unimplemented!()
}

#[deprecated]
pub fn take_if_file_handle<T: BufRead>() -> impl Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode> {
    switch_err(
        |(Locatable { pos, .. }, Locatable { element, .. })| match element.parse::<u32>() {
            Ok(d) => Some(Ok(Expression::FileHandle(d.into()).at(pos))),
            Err(err) => Some(Err(err.into()).with_err_at(pos)),
        },
        and(
            take_if_symbol('#'),
            demand("Expected digits after #", number_literal::take_if_digits()),
        ),
    )
}

pub fn parenthesis<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QErrorNode>)> {
    // TODO allow skipping whitespace inside parenthesis
    map_ng(in_parenthesis(expression_node()), |v| {
        Expression::Parenthesis(Box::new(v))
    })
}

#[deprecated]
fn take_if_parenthesis<T: BufRead + 'static>() -> impl Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>
{
    // TODO allow skipping whitespace inside parenthesis
    apply(
        |(open_parenthesis_pos, x)| Expression::Parenthesis(Box::new(x)).at(open_parenthesis_pos),
        between('(', ')', take_if_expression_node()),
    )
}

mod string_literal {
    use super::*;

    pub fn string_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QErrorNode>)> {
        map_ng(
            if_first_demand_second(
                if_first_maybe_second(try_read_char('"'), read_any_str_while(|ch| ch != '"')),
                try_read_char('"'),
                || QError::SyntaxError("Unterminated string".to_string()),
            ),
            |((_, opt_str), _)| Expression::StringLiteral(opt_str.unwrap_or_default()),
        )
    }

    #[deprecated]
    pub fn take_if_string_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode>> {
        apply(
            |(l, (string_lexemes, _))| {
                let pos = l.pos();
                let text = string_lexemes.into_iter().fold(
                    String::new(),
                    |acc, Locatable { element, .. }| {
                        format!("{}{}", acc, element) // concatenate strings
                    },
                );
                Expression::StringLiteral(text).at(pos)
            },
            and(
                take_if_symbol('"'),
                and(
                    take_until(|x: &LexemeNode| x.is_eol() || x.is_symbol('"')),
                    demand("Unterminated string", take_if_symbol('"')),
                ),
            ),
        )
    }
}

mod number_literal {
    use super::*;

    pub fn number_literal<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
        map_to_result_no_undo(
            if_first_maybe_second(
                with_pos(read_any_digits()),
                if_first_maybe_second(
                    if_first_demand_second(try_read_char('.'), read_any_digits(), || {
                        QError::SyntaxError("Expected digits after decimal point".to_string())
                    }),
                    try_read_char('#'),
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

    #[deprecated]
    pub fn take_if_number_literal<T: BufRead + 'static>(
    ) -> impl Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode> {
        switch_err(
            |(l, opt_r)| {
                let Locatable {
                    element: int_part_as_string,
                    pos,
                } = l;
                match opt_r {
                    Some((_, frac_part_as_string, is_double)) => parse_floating_point_literal(
                        int_part_as_string,
                        frac_part_as_string,
                        is_double,
                        pos,
                    )
                    .map(|x| Some(x))
                    .transpose(),
                    None => integer_literal_to_expression_node(int_part_as_string, pos)
                        .map(|x| Some(x))
                        .transpose(),
                }
            },
            zip_allow_right_none(
                take_if_digits(), // integer digits
                take_if_frac_part(),
            ),
        )
    }

    pub fn float_without_leading_zero<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<ExpressionNode, QErrorNode>)> {
        map_to_result_no_undo(
            if_first_maybe_second(
                if_first_demand_second(with_pos(try_read_char('.')), read_any_digits(), || {
                    QError::SyntaxError("Expected digits after decimal point".to_string())
                }),
                try_read_char('#'),
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

    #[deprecated]
    pub fn take_if_float_without_leading_zero<T: BufRead + 'static>(
    ) -> impl Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode> {
        switch_err(
            |(pos, frac_part_as_string, is_double)| {
                parse_floating_point_literal(String::from("0"), frac_part_as_string, is_double, pos)
                    .map(|x| Some(x))
                    .transpose()
            },
            take_if_frac_part(),
        )
    }

    #[deprecated]
    fn take_if_frac_part<T: BufRead + 'static>(
    ) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<(Location, String, bool)>> {
        apply(
            |(l, r)| (l.pos(), r.0, r.1.is_some()),
            and(
                take_if_symbol('.'),
                zip_allow_right_none(
                    demand(
                        "Expected digits after decimal point",
                        drop_location(take_if_digits()),
                    ),
                    take_if_symbol('#'),
                ),
            ),
        )
    }

    #[deprecated]
    pub fn take_if_digits<T: BufRead>() -> impl Fn(&mut BufLexer<T>) -> OptRes<Locatable<String>> {
        take_if_map(|x: LexemeNode| match x {
            Locatable {
                element: Lexeme::Digits(digits),
                pos,
            } => Some(digits.at(pos)),
            _ => None,
        })
    }

    fn integer_literal_to_expression_node(
        s: String,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
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
            Err(e) => Err(e.into()).with_err_at(pos),
        }
    }

    fn parse_floating_point_literal(
        integer_digits: String,
        fraction_digits: String,
        is_double: bool,
        pos: Location,
    ) -> Result<ExpressionNode, QErrorNode> {
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f).at(pos)),
                Err(err) => Err(err.into()).with_err_at(pos),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f).at(pos)),
                Err(err) => Err(err.into()).with_err_at(pos),
            }
        }
    }
}

mod word {
    use super::*;

    pub fn word<T: BufRead + 'static>(
    ) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Expression, QErrorNode>)> {
        map_ng(
            if_first_maybe_second(
                name::name(),
                in_parenthesis(csv_zero_or_more(expression_node())),
            ),
            |(n, opt_v)| match opt_v {
                Some(v) => Expression::FunctionCall(n, v),
                None => Expression::VariableName(n),
            },
        )
    }

    #[deprecated]
    pub fn take_if_word<T: BufRead + 'static>(
    ) -> impl Fn(&mut BufLexer<T>) -> OptRes<ExpressionNode> {
        switch_err(
            |(name_node, opt_r)| {
                match opt_r {
                    // found opening parenthesis e.g. Foo() or Foo(12, 43)
                    Some((p, r)) => {
                        if r.is_empty() {
                            Some(
                                Err(QError::SyntaxError("Expected expression".to_string()))
                                    .with_err_at(p),
                            )
                        } else {
                            Some(Ok(name_node.map(|n| Expression::FunctionCall(n, r))))
                        }
                    }
                    // no opening parenthesis e.g. A$ or A
                    None => Some(Ok(name_node.map(|n| Expression::VariableName(n)))),
                }
            },
            zip_allow_right_none(
                name::take_if_name_node(),
                between('(', ')', csv(super::take_if_expression_node())),
            ),
        )
    }
}

pub fn operand<T: BufRead + 'static>(
    had_parenthesis_before: bool,
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Locatable<Operand>, QErrorNode>)> {
    or_vec_ng(vec![
        skipping_whitespace_ng(with_pos(lte())),
        skipping_whitespace_ng(with_pos(gte())),
        map_ng(skipping_whitespace_ng(with_pos(try_read_char('='))), |x| {
            x.map(|_| Operand::Equal)
        }),
        map_ng(skipping_whitespace_ng(with_pos(try_read_char('+'))), |x| {
            x.map(|_| Operand::Plus)
        }),
        map_ng(skipping_whitespace_ng(with_pos(try_read_char('-'))), |x| {
            x.map(|_| Operand::Minus)
        }),
        map_ng(skipping_whitespace_ng(with_pos(try_read_char('*'))), |x| {
            x.map(|_| Operand::Multiply)
        }),
        map_ng(skipping_whitespace_ng(with_pos(try_read_char('/'))), |x| {
            x.map(|_| Operand::Divide)
        }),
        if had_parenthesis_before {
            // skip whitespace + AND
            map_ng(
                skipping_whitespace_ng(with_pos(try_read_keyword(Keyword::And))),
                |x| x.map(|_| Operand::And),
            )
        } else {
            // demand whitespace + AND
            map_ng(
                and_ng(
                    read_any_whitespace(),
                    with_pos(try_read_keyword(Keyword::And)),
                ),
                |(_, x)| x.map(|_| Operand::And),
            )
        },
        if had_parenthesis_before {
            // skip whitespace + OR
            map_ng(
                skipping_whitespace_ng(with_pos(try_read_keyword(Keyword::Or))),
                |x| x.map(|_| Operand::Or),
            )
        } else {
            // demand whitespace + OR
            map_ng(
                and_ng(
                    read_any_whitespace(),
                    with_pos(try_read_keyword(Keyword::Or)),
                ),
                |(_, x)| x.map(|_| Operand::Or),
            )
        },
    ])
}

fn lte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Operand, QErrorNode>)> {
    map_to_result_no_undo(
        if_first_maybe_second(
            try_read_char('<'),
            with_pos(read_any_char_if(|ch| ch == '=' || ch == '>')),
        ),
        |(_, opt_r)| match opt_r {
            Some(Locatable { element: '=', .. }) => Ok(Operand::LessOrEqual),
            Some(Locatable { element: '>', .. }) => Ok(Operand::NotEqual),
            None => Ok(Operand::Less),
            Some(Locatable { element, pos }) => Err(QError::SyntaxError(format!(
                "Invalid character {} after <",
                element
            )))
            .with_err_at(pos),
        },
    )
}

fn gte<T: BufRead + 'static>(
) -> Box<dyn Fn(EolReader<T>) -> (EolReader<T>, Result<Operand, QErrorNode>)> {
    map_ng(
        if_first_maybe_second(try_read_char('>'), try_read_char('=')),
        |(_, opt_r)| match opt_r {
            Some(_) => Operand::GreaterOrEqual,
            _ => Operand::Greater,
        },
    )
}

#[deprecated]
fn take_if_operand<T: BufRead + 'static>(
    left_side: &ExpressionNode,
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<Locatable<Operand>>> {
    let left_side_parenthesis = left_side.is_parenthesis();
    or_vec(vec![
        or_vec(vec![
            // LTE, NE
            apply(
                |(l, r)| {
                    (match r {
                        Some('=') => Operand::LessOrEqual,
                        Some('>') => Operand::NotEqual,
                        _ => Operand::Less,
                    })
                    .at(l.pos())
                },
                zip_allow_right_none(
                    skipping_whitespace(take_if_symbol('<')),
                    or(
                        drop_location(take_if_symbol('=')),
                        drop_location(take_if_symbol('>')),
                    ),
                ),
            ),
            // GTE
            apply(
                |(l, r)| {
                    (match r {
                        Some('=') => Operand::GreaterOrEqual,
                        _ => Operand::Greater,
                    })
                    .at(l.pos())
                },
                zip_allow_right_none(
                    skipping_whitespace(take_if_symbol('>')),
                    drop_location(take_if_symbol('=')),
                ),
            ),
            take_if_simple_op('=', Operand::Equal),
            take_if_simple_op('+', Operand::Plus),
            take_if_simple_op('-', Operand::Minus),
            take_if_simple_op('*', Operand::Multiply),
            take_if_simple_op('/', Operand::Divide),
        ]),
        // AND
        take_and_or_op(Keyword::And, Operand::And, left_side_parenthesis),
        take_and_or_op(Keyword::Or, Operand::Or, left_side_parenthesis),
    ])
}

#[deprecated]
fn take_if_simple_op<T: BufRead + 'static>(
    ch: char,
    op: Operand,
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<Locatable<Operand>>> {
    Box::new(map_locatable(
        move |_| op,
        skipping_whitespace(take_if_symbol(ch)),
    ))
}

#[deprecated]
fn take_and_or_op<T: BufRead + 'static>(
    k: Keyword,
    op: Operand,
    left_side_parenthesis: bool,
) -> Box<dyn Fn(&mut BufLexer<T>) -> OptRes<Locatable<Operand>>> {
    Box::new(switch(
        move |(had_whitespace, r)| {
            if had_whitespace.is_some() || left_side_parenthesis {
                Some(op.at(r.pos()))
            } else {
                None
            }
        },
        zip_allow_left_none(take_if_whitespace(), take_if_keyword(k)),
    ))
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
            QError::SyntaxError("Expected top level token".to_string())
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
            QError::SyntaxError("Expected top level token".to_string())
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
