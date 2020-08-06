use crate::common::*;
use crate::lexer::{BufLexer, Keyword, LexemeNode};
use crate::parser::buf_lexer::*;
use crate::parser::type_qualifier;
use crate::parser::{
    unexpected, Expression, ExpressionNode, Name, Operand, ParserError, UnaryOperand,
};
use crate::variant;
use std::io::BufRead;

pub fn try_read<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<ExpressionNode>, ParserError> {
    let opt_first = try_single_expression(lexer)?;
    match opt_first {
        Some(first) => try_parse_second_expression(lexer, first)
            .map(|x| x.simplify_unary_minus_literals())
            .map(|x| Some(x)),
        None => Ok(None),
    }
}

fn try_single_expression<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Option<ExpressionNode>, ParserError> {
    let next = lexer.peek()?;
    match next {
        LexemeNode::Symbol('"', _) => string_literal::read(lexer).map(|x| Some(x)),
        LexemeNode::Word(_, _) => word::read(lexer).map(|x| Some(x)),
        LexemeNode::Digits(_, _) => number_literal::read(lexer).map(|x| Some(x)),
        LexemeNode::Symbol('.', _) => number_literal::read_dot_float(lexer).map(|x| Some(x)),
        LexemeNode::Symbol('-', minus_pos) => {
            lexer.read()?;
            let child = demand(lexer, try_read, "Expected expression after unary minus")?;
            Ok(Some(apply_unary_priority_order(
                child,
                UnaryOperand::Minus,
                minus_pos,
            )))
        }
        LexemeNode::Keyword(Keyword::Not, _, not_pos) => {
            lexer.read()?;
            read_demand_whitespace(lexer, "Expected whitespace after NOT")?;
            let child = demand(lexer, try_read, "Expected expression after NOT")?;
            Ok(Some(apply_unary_priority_order(
                child,
                UnaryOperand::Not,
                not_pos,
            )))
        }
        LexemeNode::Symbol('(', pos) => {
            lexer.read()?;
            let inner = demand_skipping_whitespace(
                lexer,
                try_read,
                "Expected expression inside parenthesis",
            )?;
            skip_whitespace(lexer)?;
            let closing = lexer.read()?;
            match closing {
                LexemeNode::Symbol(')', _) => {
                    Ok(Some(Expression::Parenthesis(Box::new(inner)).at(pos)))
                }
                _ => unexpected("Expected closing parenthesis", closing),
            }
        }
        LexemeNode::Symbol('#', pos) => {
            // file handle e.g. CLOSE #1
            lexer.read()?;
            let digits = demand_digits(lexer)?;
            match digits.parse::<u32>() {
                Ok(d) => Ok(Some(Expression::FileHandle(d.into()).at(pos))),
                Err(err) => Err(ParserError::Internal(err.to_string(), pos)),
            }
        }
        _ => Ok(None),
    }
}

fn try_parse_second_expression<T: BufRead>(
    lexer: &mut BufLexer<T>,
    left_side: ExpressionNode,
) -> Result<ExpressionNode, ParserError> {
    let operand = try_parse_operand(lexer, &left_side)?;
    match operand {
        Some((op, pos)) => {
            let right_side =
                demand_skipping_whitespace(lexer, try_read, "Expected right side expression")?;
            Ok(apply_priority_order(left_side, right_side, op, pos))
        }
        None => Ok(left_side),
    }
}

fn apply_priority_order(
    left_side: ExpressionNode,
    right_side: ExpressionNode,
    op: Operand,
    pos: Location,
) -> ExpressionNode {
    match right_side.as_ref() {
        Expression::BinaryExpression(r_op, r_left, r_right) => {
            let should_flip = op.is_arithmetic() && (r_op.is_relational() || r_op.is_binary())
                || op.is_relational() && r_op.is_binary()
                || op == Operand::And && *r_op == Operand::Or
                || (op == Operand::Multiply || op == Operand::Divide)
                    && (*r_op == Operand::Plus || *r_op == Operand::Minus);
            if should_flip {
                Expression::BinaryExpression(
                    *r_op,
                    Box::new(
                        Expression::BinaryExpression(op, Box::new(left_side), r_left.clone())
                            .at(pos),
                    ),
                    r_right.clone(),
                )
                .at(right_side.location())
            } else {
                Expression::BinaryExpression(op, Box::new(left_side), Box::new(right_side)).at(pos)
            }
        }
        _ => Expression::BinaryExpression(op, Box::new(left_side), Box::new(right_side)).at(pos),
    }
}

fn apply_unary_priority_order(
    child: ExpressionNode,
    op: UnaryOperand,
    pos: Location,
) -> ExpressionNode {
    match child.as_ref() {
        Expression::BinaryExpression(r_op, r_left, r_right) => {
            let should_flip = op == UnaryOperand::Minus || r_op.is_binary();
            if should_flip {
                Expression::BinaryExpression(
                    *r_op,
                    Box::new(Expression::UnaryExpression(op, r_left.clone()).at(pos)),
                    r_right.clone(),
                )
                .at(child.location())
            } else {
                Expression::UnaryExpression(op, Box::new(child)).at(pos)
            }
        }
        _ => Expression::UnaryExpression(op, Box::new(child)).at(pos),
    }
}

/// Parses a comma separated list of expressions.
fn parse_expression_list_with_parentheses<T: BufRead>(
    lexer: &mut BufLexer<T>,
) -> Result<Vec<ExpressionNode>, ParserError> {
    let mut args: Vec<ExpressionNode> = vec![];
    const STATE_OPEN_PARENTHESIS: u8 = 0;
    const STATE_CLOSE_PARENTHESIS: u8 = 1;
    const STATE_COMMA: u8 = 2;
    const STATE_EXPRESSION: u8 = 3;
    let mut state = STATE_OPEN_PARENTHESIS;
    while state != STATE_CLOSE_PARENTHESIS {
        skip_whitespace(lexer)?;
        let next = lexer.peek()?;
        match next {
            LexemeNode::Symbol(')', _) => {
                lexer.read()?;
                if state == STATE_EXPRESSION {
                    state = STATE_CLOSE_PARENTHESIS;
                } else if state == STATE_OPEN_PARENTHESIS {
                    return unexpected("Expected expression", next);
                } else {
                    return unexpected("Expected expression after comma", next);
                }
            }
            LexemeNode::Symbol(',', _) => {
                lexer.read()?;
                if state == STATE_EXPRESSION {
                    state = STATE_COMMA;
                } else {
                    return unexpected("Unexpected comma", next);
                }
            }
            LexemeNode::EOL(_, _) | LexemeNode::EOF(_) => {
                return unexpected("Premature end of expression list", next);
            }
            _ => {
                if state == STATE_EXPRESSION {
                    return unexpected("Expected comma or )", next);
                }
                args.push(demand(lexer, try_read, "Expected expression")?);
                state = STATE_EXPRESSION;
            }
        }
    }
    Ok(args)
}

mod string_literal {
    use super::*;
    pub fn read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<ExpressionNode, ParserError> {
        let mut buf: String = String::new();
        let pos = lexer.read()?.location(); // read double quote
                                            // read until we hit the next double quote
        loop {
            let l = lexer.read()?;
            match l {
                LexemeNode::EOF(_) => return unexpected("EOF while looking for end of string", l),
                LexemeNode::EOL(_, _) => {
                    return unexpected("Unexpected new line while looking for end of string", l)
                }
                LexemeNode::Keyword(_, s, _)
                | LexemeNode::Word(s, _)
                | LexemeNode::Whitespace(s, _) => buf.push_str(&s),
                LexemeNode::Symbol(c, _) => {
                    if c == '"' {
                        break;
                    } else {
                        buf.push(c);
                    }
                }
                LexemeNode::Digits(d, _) => buf.push_str(&format!("{}", d)),
            }
        }

        Ok(Expression::StringLiteral(buf).at(pos))
    }
}

fn demand_digits<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<String, ParserError> {
    let next = lexer.read()?;
    match next {
        LexemeNode::Digits(digits, _) => Ok(digits),
        _ => unexpected("Expected digits", next),
    }
}

mod number_literal {
    use super::*;
    pub fn read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<ExpressionNode, ParserError> {
        // consume digits
        let (digits, pos) = lexer.read()?.consume_digits();
        let found_decimal_point = skip_if(lexer, |lexeme| lexeme.is_symbol('.'))?;
        if found_decimal_point {
            parse_floating_point_literal(lexer, digits, pos)
        } else {
            // no decimal point, just integer
            integer_literal_to_expression_node(digits, pos)
        }
    }

    pub fn read_dot_float<T: BufRead>(
        lexer: &mut BufLexer<T>,
    ) -> Result<ExpressionNode, ParserError> {
        let pos = lexer.read()?.location(); // consume . of .10
        parse_floating_point_literal(lexer, "0".to_string(), pos)
    }

    fn integer_literal_to_expression_node(
        s: String,
        pos: Location,
    ) -> Result<ExpressionNode, ParserError> {
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
            Err(e) => Err(ParserError::Internal(e.to_string(), pos)),
        }
    }

    fn parse_floating_point_literal<T: BufRead>(
        lexer: &mut BufLexer<T>,
        integer_digits: String,
        pos: Location,
    ) -> Result<ExpressionNode, ParserError> {
        let fraction_digits = demand_digits(lexer)?;
        let is_double = skip_if(lexer, |lexeme| lexeme.is_symbol('#'))?;
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f).at(pos)),
                Err(err) => Err(ParserError::Internal(err.to_string(), pos)),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f).at(pos)),
                Err(err) => Err(ParserError::Internal(err.to_string(), pos)),
            }
        }
    }
}

mod word {
    use super::*;
    pub fn read<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<ExpressionNode, ParserError> {
        // is it maybe a qualified variable name
        let (word, pos) = lexer.read()?.consume_word();
        let qualifier = type_qualifier::try_read(lexer)?;
        let name = Name::new(word.into(), qualifier);
        // it could be a function call?
        let found_opening_parenthesis = skip_if(lexer, |lexeme| lexeme.is_symbol('('))?;
        if found_opening_parenthesis {
            let args = parse_expression_list_with_parentheses(lexer)?;
            Ok(Expression::FunctionCall(name, args).at(pos))
        } else {
            Ok(Expression::VariableName(name).at(pos))
        }
    }
}

fn try_parse_operand<T: BufRead>(
    lexer: &mut BufLexer<T>,
    left_side: &ExpressionNode,
) -> Result<Option<(Operand, Location)>, ParserError> {
    // if we can't find an operand, we need to restore the whitespace as it was,
    // in case there is a next call that will be demanding for it
    lexer.begin_transaction();
    let (opt_space, next) = read_preserve_whitespace(lexer)?;
    match next {
        LexemeNode::Symbol('<', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((less_or_lte_or_ne(lexer)?, pos)))
        }
        LexemeNode::Symbol('>', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((greater_or_gte(lexer)?, pos)))
        }
        LexemeNode::Symbol('=', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((Operand::Equal, pos)))
        }
        LexemeNode::Symbol('+', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((Operand::Plus, pos)))
        }
        LexemeNode::Symbol('-', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((Operand::Minus, pos)))
        }
        LexemeNode::Symbol('*', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((Operand::Multiply, pos)))
        }
        LexemeNode::Symbol('/', pos) => {
            lexer.commit_transaction()?;
            Ok(Some((Operand::Divide, pos)))
        }
        LexemeNode::Keyword(Keyword::And, _, pos) => {
            if opt_space.is_some() || left_side.is_parenthesis() {
                lexer.commit_transaction()?;
                Ok(Some((Operand::And, pos)))
            } else {
                lexer.rollback_transaction()?;
                Ok(None)
            }
        }
        LexemeNode::Keyword(Keyword::Or, _, pos) => {
            if opt_space.is_some() || left_side.is_parenthesis() {
                lexer.commit_transaction()?;
                Ok(Some((Operand::Or, pos)))
            } else {
                lexer.rollback_transaction()?;
                Ok(None)
            }
        }
        _ => {
            lexer.rollback_transaction()?;
            Ok(None)
        }
    }
}

fn less_or_lte_or_ne<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Operand, ParserError> {
    let next = lexer.peek()?;
    match next {
        LexemeNode::Symbol('=', _) => {
            lexer.read()?;
            Ok(Operand::LessOrEqual)
        }
        LexemeNode::Symbol('>', _) => {
            lexer.read()?;
            Ok(Operand::NotEqual)
        }
        _ => Ok(Operand::Less),
    }
}

fn greater_or_gte<T: BufRead>(lexer: &mut BufLexer<T>) -> Result<Operand, ParserError> {
    let x = skip_if(lexer, |lexeme| lexeme.is_symbol('=')).map(|found_equal_sign| {
        if found_equal_sign {
            Operand::GreaterOrEqual
        } else {
            Operand::Greater
        }
    })?;
    Ok(x)
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::lexer::{Keyword, LexemeNode};
    use crate::parser::{Expression, Name, Operand, ParserError, Statement, UnaryOperand};

    macro_rules! assert_expression {
        ($left:expr, $right:expr) => {
            let program = parse(&format!("PRINT {}", $left)).demand_single_statement();
            match program {
                Statement::SubCall(_, args) => {
                    assert_eq!(1, args.len());
                    assert_eq!(args[0].clone().strip_location(), $right);
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
        assert_literal_expression!("\"hello, world\"", "hello, world");
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
                ParserError::Unexpected(
                    "Expected expression".to_string(),
                    LexemeNode::Symbol(')', Location::new(1, 15))
                )
            );
        }

        #[test]
        fn test_function_call_qualified_expression_no_args() {
            assert_eq!(
                parse_err("PRINT IsValid%()"),
                ParserError::Unexpected(
                    "Expected expression".to_string(),
                    LexemeNode::Symbol(')', Location::new(1, 16))
                )
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
            ParserError::Unterminated(LexemeNode::Keyword(
                Keyword::And,
                "AND".to_string(),
                Location::new(1, 8)
            ))
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
            ParserError::Unterminated(LexemeNode::Keyword(
                Keyword::Or,
                "OR".to_string(),
                Location::new(1, 8)
            ))
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
