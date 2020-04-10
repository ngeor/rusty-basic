use super::parse_result::ParseResult;
use super::*;
use crate::lexer::LexemeNode;
use std::convert::TryFrom;

impl From<ExpressionNode> for ParseResult<ExpressionNode> {
    fn from(expr: ExpressionNode) -> ParseResult<ExpressionNode> {
        ParseResult::Match(expr)
    }
}

impl<T: BufRead> Parser<T> {
    pub fn demand_expression(&mut self) -> Result<ExpressionNode, LexerError> {
        match self.try_parse_expression() {
            Ok(x) => x.demand("Expected expression"),
            Err(err) => Err(err),
        }
    }

    pub fn try_parse_expression(&mut self) -> Result<ParseResult<ExpressionNode>, LexerError> {
        let first = self._try_parse_single_expression()?;
        match first {
            ParseResult::Match(e) => self
                ._try_parse_second_expression(e)
                .map(|x| ParseResult::Match(x)),
            _ => Ok(first),
        }
    }

    fn _try_parse_single_expression(&mut self) -> Result<ParseResult<ExpressionNode>, LexerError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol('"', _) => {
                self._parse_string_literal().map(|x| ParseResult::Match(x))
            }
            LexemeNode::Word(word, pos) => {
                self._parse_word(&word, pos).map(|x| ParseResult::Match(x))
            }
            LexemeNode::Digits(digits, pos) => self
                ._parse_number_literal(digits, pos)
                .map(|x| ParseResult::Match(x)),
            LexemeNode::Symbol('.', pos) => self
                ._parse_floating_point_literal(0, pos)
                .map(|x| ParseResult::Match(x)),
            LexemeNode::Symbol('-', minus_pos) => {
                self.buf_lexer.consume();
                let opt_child = self._try_parse_single_expression()?;
                Ok(match opt_child {
                    ParseResult::Match(e) => ParseResult::Match(ExpressionNode::unary_minus(
                        Locatable::new(UnaryOperand::Minus, minus_pos),
                        e,
                    )),
                    _ => opt_child,
                })
            }
            _ => Ok(ParseResult::NoMatch(next)),
        }
    }

    fn _try_parse_second_expression(
        &mut self,
        left_side: ExpressionNode,
    ) -> Result<ExpressionNode, LexerError> {
        let operand = self._try_parse_operand()?;
        match operand {
            Some(op) => {
                self.buf_lexer.skip_whitespace()?;
                let right_side = self.demand_expression()?;
                Ok(ExpressionNode::BinaryExpression(
                    op,
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            None => Ok(left_side),
        }
    }

    /// Parses a comma separated list of expressions.
    pub fn parse_expression_list(&mut self) -> Result<Vec<ExpressionNode>, LexerError> {
        let mut args: Vec<ExpressionNode> = vec![];
        let optional_first_arg = self.try_parse_expression()?;
        if let ParseResult::Match(first_arg) = optional_first_arg {
            args.push(first_arg);
            while self._read_comma_between_arguments()? {
                self.buf_lexer.skip_whitespace()?;
                let next_arg = self.demand_expression()?;
                args.push(next_arg);
            }
        }

        Ok(args)
    }

    fn _read_comma_between_arguments(&mut self) -> Result<bool, LexerError> {
        // skip whitespace after previous arg
        self.buf_lexer.skip_whitespace()?;
        self.buf_lexer.try_consume_symbol(',').map(|x| x.is_some())
    }

    fn _parse_string_literal(&mut self) -> Result<ExpressionNode, LexerError> {
        // verify we read a double quote
        let pos = self.buf_lexer.demand_symbol('"')?;

        let mut buf: String = String::new();

        // read until we hit the next double quote
        loop {
            let l = self.buf_lexer.read()?;
            self.buf_lexer.consume();
            match l {
                LexemeNode::Symbol('"', _) => break,
                LexemeNode::EOF(_) => {
                    return Err(LexerError::Unexpected(
                        format!("EOF while looking for end of string"),
                        l,
                    ))
                }
                LexemeNode::EOL(_, _) => {
                    return Err(LexerError::Unexpected(
                        format!("Unexpected new line while looking for end of string"),
                        l,
                    ))
                }
                _ => l.strip_location().push_to(&mut buf),
            }
        }

        Ok(ExpressionNode::StringLiteral(buf, pos))
    }

    fn _parse_number_literal(
        &mut self,
        digits: u32,
        pos: Location,
    ) -> Result<ExpressionNode, LexerError> {
        // consume digits
        self.buf_lexer.consume();
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Symbol('.', _) => self._parse_floating_point_literal(digits, pos),
            _ => {
                // no decimal point, just integer
                match i32::try_from(digits) {
                    Ok(i) => Ok(ExpressionNode::IntegerLiteral(i, pos)),
                    Err(err) => Err(LexerError::Internal(err.to_string(), pos)),
                }
            }
        }
    }

    fn _demand_digits(&mut self) -> Result<u32, LexerError> {
        let next = self.buf_lexer.read()?;
        match next {
            LexemeNode::Digits(digits, _) => {
                self.buf_lexer.consume();
                Ok(digits)
            }
            _ => Err(LexerError::Unexpected(format!("Expected digits"), next)),
        }
    }

    fn _parse_floating_point_literal(
        &mut self,
        integer_digits: u32,
        pos: Location,
    ) -> Result<ExpressionNode, LexerError> {
        // consume dot
        self.buf_lexer.consume();
        let fraction_digits = self._demand_digits()?;
        let is_double = self.buf_lexer.try_consume_symbol('#')?.is_some();
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(ExpressionNode::DoubleLiteral(f, pos)),
                Err(err) => Err(LexerError::Internal(err.to_string(), pos)),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(ExpressionNode::SingleLiteral(f, pos)),
                Err(err) => Err(LexerError::Internal(err.to_string(), pos)),
            }
        }
    }

    fn _parse_word(&mut self, word: &String, pos: Location) -> Result<ExpressionNode, LexerError> {
        self.buf_lexer.consume();
        // is it maybe a qualified variable name
        let qualifier = self.try_parse_type_qualifier()?;
        // it could be a function call?
        if self.buf_lexer.try_consume_symbol('(')?.is_some() {
            let args = self.parse_expression_list()?;
            self.buf_lexer.demand_symbol(')')?;
            Ok(ExpressionNode::FunctionCall(
                NameNode::new(word, qualifier, pos),
                args,
            ))
        } else {
            Ok(ExpressionNode::VariableName(NameNode::new(
                word, qualifier, pos,
            )))
        }
    }

    fn _try_parse_operand(&mut self) -> Result<Option<OperandNode>, LexerError> {
        self.buf_lexer.mark();
        self.buf_lexer.skip_whitespace()?;
        match self
            .buf_lexer
            .try_consume_symbol_one_of(vec!['<', '+', '-'])?
        {
            Some((ch, pos)) => {
                self.buf_lexer.clear();
                if ch == '<' {
                    Ok(Some(Locatable::new(self._less_or_lte()?, pos)))
                } else if ch == '+' {
                    Ok(Some(Locatable::new(Operand::Plus, pos)))
                } else if ch == '-' {
                    Ok(Some(Locatable::new(Operand::Minus, pos)))
                } else {
                    panic!(format!("Unexpected symbol {}", ch))
                }
            }
            None => {
                self.buf_lexer.backtrack();
                Ok(None)
            }
        }
    }

    fn _less_or_lte(&mut self) -> Result<Operand, LexerError> {
        if self.buf_lexer.try_consume_symbol('=')?.is_some() {
            Ok(Operand::LessOrEqualThan)
        } else {
            Ok(Operand::LessThan)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_parse_literal<TResult>(input: &str, expected: TResult)
    where
        TResult: std::fmt::Display + PartialEq,
        Expression: From<TResult>,
    {
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(expression, Expression::from(expected));
    }

    #[test]
    fn test_literals() {
        assert_parse_literal("\"hello, world\"", "hello, world");
        assert_parse_literal("42", 42);
        assert_parse_literal("4.2", 4.2_f32);
        assert_parse_literal("0.5", 0.5_f32);
        assert_parse_literal(".5", 0.5_f32);
        assert_parse_literal("3.14#", 3.14_f64);
        assert_parse_literal("-42", -42);
    }

    #[test]
    fn test_variable_expression() {
        let input = "A";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(expression, Expression::VariableName(Name::from("A")));
    }

    #[test]
    fn test_function_call_expression_no_args() {
        let input = "IsValid()";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::FunctionCall(Name::from("IsValid"), vec![])
        );
    }

    #[test]
    fn test_function_call_qualified_expression_no_args() {
        let input = "IsValid%()";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::FunctionCall(Name::from("IsValid%"), vec![])
        );
    }

    #[test]
    fn test_function_call_expression_one_arg() {
        let input = "IsValid(42)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::FunctionCall(Name::from("IsValid"), vec![Expression::IntegerLiteral(42)])
        );
    }

    #[test]
    fn test_function_call_expression_two_args() {
        let input = "CheckProperty(42, \"age\")";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                Name::from("CheckProperty"),
                vec![Expression::from(42), Expression::from("age")]
            )
        );
    }

    #[test]
    fn test_function_call_in_function_call() {
        let input = "CheckProperty(LookupName(\"age\"), Confirm())";
        let mut parser = Parser::from(input);
        let expression_node = parser.demand_expression().unwrap();
        assert_eq!(
            expression_node,
            ExpressionNode::FunctionCall(
                NameNode::from("CheckProperty").at(Location::new(1, 1)),
                vec![
                    ExpressionNode::FunctionCall(
                        NameNode::from("LookupName").at(Location::new(1, 15)),
                        vec![ExpressionNode::StringLiteral(
                            "age".to_string(),
                            Location::new(1, 26)
                        )]
                    ),
                    ExpressionNode::FunctionCall(
                        NameNode::from("Confirm").at(Location::new(1, 34)),
                        vec![]
                    )
                ]
            )
        );
    }

    #[test]
    fn test_lte() {
        let input = "N <= 1";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::LessOrEqualThan,
                Box::new(Expression::variable_name("N")),
                Box::new(Expression::IntegerLiteral(1)),
            ),
        );
    }

    #[test]
    fn test_less_than() {
        let input = "A < B";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::LessThan,
                Box::new(Expression::variable_name("A")),
                Box::new(Expression::variable_name("B")),
            ),
        );
    }

    #[test]
    fn test_plus() {
        let input = "N + 1";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Plus,
                Box::new(Expression::variable_name("N")),
                Box::new(Expression::IntegerLiteral(1)),
            ),
        );
    }

    #[test]
    fn test_minus() {
        let input = "N - 2";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Minus,
                Box::new(Expression::variable_name("N")),
                Box::new(Expression::IntegerLiteral(2)),
            ),
        );
    }

    #[test]
    fn test_negated_variable() {
        let input = "-N";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::UnaryExpression(
                UnaryOperand::Minus,
                Box::new(Expression::variable_name("N"))
            ),
        );
    }

    #[test]
    fn test_fib_expression() {
        let input = "Fib(N - 1) + Fib(N - 2)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Plus,
                Box::new(Expression::FunctionCall(
                    Name::from("Fib"),
                    vec![Expression::BinaryExpression(
                        Operand::Minus,
                        Box::new(Expression::variable_name("N")),
                        Box::new(Expression::IntegerLiteral(1)),
                    )],
                )),
                Box::new(Expression::FunctionCall(
                    Name::from("Fib"),
                    vec![Expression::minus(
                        Expression::variable_name("N"),
                        Expression::IntegerLiteral(2),
                    )],
                )),
            ),
        );
    }

    #[test]
    fn test_negated_function_call() {
        let input = "-Fib(-N)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap().strip_location();
        assert_eq!(
            expression,
            Expression::UnaryExpression(
                UnaryOperand::Minus,
                Box::new(Expression::FunctionCall(
                    Name::from("Fib"),
                    vec![Expression::UnaryExpression(
                        UnaryOperand::Minus,
                        Box::new(Expression::variable_name("N")),
                    )],
                ))
            )
        );
    }
}
