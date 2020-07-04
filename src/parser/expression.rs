use crate::common::*;
use crate::lexer::{Keyword, LexemeNode};
use crate::parser::{
    unexpected, Expression, ExpressionNode, Name, Operand, Parser, ParserError, UnaryOperand,
};
use crate::variant;
use std::io::BufRead;

impl<T: BufRead> Parser<T> {
    pub fn read_demand_expression_skipping_whitespace(
        &mut self,
    ) -> Result<ExpressionNode, ParserError> {
        let next = self.read_skipping_whitespace()?;
        self.demand_expression(next)
    }

    pub fn read_demand_expression(&mut self) -> Result<ExpressionNode, ParserError> {
        let next = self.buf_lexer.read()?;
        self.demand_expression(next)
    }

    pub fn demand_expression(&mut self, next: LexemeNode) -> Result<ExpressionNode, ParserError> {
        let first = self.demand_single_expression(next)?;
        self.try_parse_second_expression(first)
    }

    fn demand_single_expression(
        &mut self,
        next: LexemeNode,
    ) -> Result<ExpressionNode, ParserError> {
        match next {
            LexemeNode::Symbol('"', pos) => self.parse_string_literal(pos),
            LexemeNode::Word(word, pos) => self.parse_word(word, pos),
            LexemeNode::Digits(digits, pos) => self.parse_number_literal(digits, pos),
            LexemeNode::Symbol('.', pos) => self.parse_floating_point_literal("0".to_string(), pos),
            LexemeNode::Symbol('-', minus_pos) => {
                let child = self.read_demand_expression()?;
                Ok(Expression::unary_minus(child).at(minus_pos))
            }
            LexemeNode::Keyword(Keyword::Not, _, not_pos) => {
                self.read_demand_whitespace("Expected whitespace after NOT")?;
                let child = self.read_demand_expression()?;
                Ok(Expression::UnaryExpression(UnaryOperand::Not, Box::new(child)).at(not_pos))
            }
            LexemeNode::Symbol('(', pos) => {
                let inner = self.read_demand_expression_skipping_whitespace()?;
                let closing = self.read_skipping_whitespace()?;
                match closing {
                    LexemeNode::Symbol(')', _) => {
                        Ok(Expression::Parenthesis(Box::new(inner)).at(pos))
                    }
                    _ => unexpected("Expected closing parenthesis", closing)
                }
            }
            _ => unexpected("Expected expression", next),
        }
    }

    fn try_parse_second_expression(
        &mut self,
        left_side: ExpressionNode,
    ) -> Result<ExpressionNode, ParserError> {
        let operand = self.try_parse_operand()?;
        match operand {
            Some((op, pos)) => {
                let right_side = self.read_demand_expression_skipping_whitespace()?;
                Ok(Self::apply_priority_order(left_side, right_side, op, pos))
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
                let should_flip = op == Operand::Plus && *r_op == Operand::Less;
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
                    Expression::BinaryExpression(op, Box::new(left_side), Box::new(right_side))
                        .at(pos)
                }
            }
            _ => {
                Expression::BinaryExpression(op, Box::new(left_side), Box::new(right_side)).at(pos)
            }
        }
    }

    /// Parses a comma separated list of expressions.
    fn parse_expression_list_with_parentheses(
        &mut self,
    ) -> Result<Vec<ExpressionNode>, ParserError> {
        let mut args: Vec<ExpressionNode> = vec![];
        const STATE_OPEN_PARENTHESIS: u8 = 0;
        const STATE_CLOSE_PARENTHESIS: u8 = 1;
        const STATE_COMMA: u8 = 2;
        const STATE_EXPRESSION: u8 = 3;
        let mut state = STATE_OPEN_PARENTHESIS;
        while state != STATE_CLOSE_PARENTHESIS {
            let next = self.read_skipping_whitespace()?;
            match next {
                LexemeNode::Symbol(')', _) => {
                    if state == STATE_OPEN_PARENTHESIS || state == STATE_EXPRESSION {
                        state = STATE_CLOSE_PARENTHESIS;
                    } else {
                        return unexpected("Expected expression after comma", next);
                    }
                }
                LexemeNode::Symbol(',', _) => {
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
                    args.push(self.demand_expression(next)?);
                    state = STATE_EXPRESSION;
                }
            }
        }
        Ok(args)
    }

    fn parse_string_literal(&mut self, pos: Location) -> Result<ExpressionNode, ParserError> {
        let mut buf: String = String::new();

        // read until we hit the next double quote
        loop {
            let l = self.buf_lexer.read()?;
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

    fn parse_number_literal(
        &mut self,
        digits: String,
        pos: Location,
    ) -> Result<ExpressionNode, ParserError> {
        // consume digits
        let found_decimal_point = self.buf_lexer.skip_if(|lexeme| lexeme.is_symbol('.'))?;
        if found_decimal_point {
            self.parse_floating_point_literal(digits, pos)
        } else {
            // no decimal point, just integer
            integer_literal_to_expression_node(digits, pos)
        }
    }

    fn demand_digits(&mut self) -> Result<String, ParserError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Digits(digits, _) => Ok(digits),
            _ => unexpected("Expected digits", next),
        }
    }

    fn parse_floating_point_literal(
        &mut self,
        integer_digits: String,
        pos: Location,
    ) -> Result<ExpressionNode, ParserError> {
        let fraction_digits = self.demand_digits()?;
        let is_double = self.buf_lexer.skip_if(|lexeme| lexeme.is_symbol('#'))?;
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

    fn parse_word(&mut self, word: String, pos: Location) -> Result<ExpressionNode, ParserError> {
        // is it maybe a qualified variable name
        let qualifier = self.try_parse_type_qualifier()?;
        // it could be a function call?
        let found_opening_parenthesis = self.buf_lexer.skip_if(|lexeme| lexeme.is_symbol('('))?;
        if found_opening_parenthesis {
            let args = self.parse_expression_list_with_parentheses()?;
            Ok(Expression::FunctionCall(Name::new(word, qualifier), args).at(pos))
        } else {
            Ok(Expression::VariableName(Name::new(word, qualifier)).at(pos))
        }
    }

    fn try_parse_operand(&mut self) -> Result<Option<(Operand, Location)>, ParserError> {
        // if we can't find an operand, we need to restore the whitespace as it was,
        // in case there is a next call that will be demanding for it
        let (opt_space, next) = self.read_preserve_whitespace()?;
        match next {
            LexemeNode::Symbol('<', pos) => Ok(Some((self.less_or_lte()?, pos))),
            LexemeNode::Symbol('+', pos) => Ok(Some((Operand::Plus, pos))),
            LexemeNode::Symbol('-', pos) => Ok(Some((Operand::Minus, pos))),
            _ => {
                self.buf_lexer.undo(next);
                match opt_space {
                    Some(x) => self.buf_lexer.undo(x),
                    _ => (),
                }
                Ok(None)
            }
        }
    }

    fn less_or_lte(&mut self) -> Result<Operand, ParserError> {
        self.buf_lexer
            .skip_if(|lexeme| lexeme.is_symbol('='))
            .map(|found_equal_sign| {
                if found_equal_sign {
                    Operand::LessOrEqual
                } else {
                    Operand::Less
                }
            })
    }
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

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;
    use crate::common::*;
    use crate::parser::{Expression, Name, Operand, Statement, UnaryOperand};

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
            assert_expression!(
                "IsValid()",
                Expression::FunctionCall(Name::from("IsValid"), vec![])
            );
        }

        #[test]
        fn test_function_call_qualified_expression_no_args() {
            assert_expression!(
                "IsValid%()",
                Expression::FunctionCall(Name::from("IsValid%"), vec![])
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
                "CheckProperty(LookupName(\"age\"), Confirm())",
                Expression::FunctionCall(
                    Name::from("CheckProperty"),
                    vec![
                        Expression::FunctionCall(
                            Name::from("LookupName"),
                            vec!["age".as_lit_expr(1, 32)]
                        )
                        .at_rc(1, 21),
                        Expression::FunctionCall(Name::from("Confirm"), vec![]).at_rc(1, 40)
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
}
