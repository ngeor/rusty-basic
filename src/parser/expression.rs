use super::*;
use crate::common::Result;
use crate::lexer::Lexeme;
use std::convert::TryFrom;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operand {
    LessOrEqualThan,
    LessThan,
    Plus,
    Minus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOperand {
    // Plus,
    Minus,
    // Not,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    VariableName(QName),
    FunctionCall(QName, Vec<Expression>),
    BinaryExpression(Operand, Box<Expression>, Box<Expression>),
    UnaryExpression(UnaryOperand, Box<Expression>),
}

impl From<f32> for Expression {
    fn from(value: f32) -> Expression {
        Expression::SingleLiteral(value)
    }
}

impl From<f64> for Expression {
    fn from(value: f64) -> Expression {
        Expression::DoubleLiteral(value)
    }
}

impl From<&str> for Expression {
    fn from(value: &str) -> Expression {
        Expression::StringLiteral(value.to_string())
    }
}

impl From<i32> for Expression {
    fn from(value: i32) -> Expression {
        Expression::IntegerLiteral(value)
    }
}

impl From<i64> for Expression {
    fn from(value: i64) -> Expression {
        Expression::LongLiteral(value)
    }
}

impl Expression {
    #[cfg(test)]
    pub fn variable_name_unqualified(name: &str) -> Expression {
        Expression::VariableName(QName::Untyped(name.to_string()))
    }

    #[cfg(test)]
    pub fn binary(operand: Operand, left: Expression, right: Expression) -> Expression {
        Expression::BinaryExpression(operand, Box::new(left), Box::new(right))
    }

    pub fn unary(operand: UnaryOperand, child: Expression) -> Expression {
        Expression::UnaryExpression(operand, Box::new(child))
    }

    pub fn unary_minus(child: Expression) -> Expression {
        match child {
            Expression::SingleLiteral(n) => Expression::from(-n),
            Expression::DoubleLiteral(n) => Expression::from(-n),
            Expression::IntegerLiteral(n) => Expression::from(-n),
            Expression::LongLiteral(n) => Expression::from(-n),
            _ => Expression::unary(UnaryOperand::Minus, child),
        }
    }

    #[cfg(test)]
    pub fn lte(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::LessOrEqualThan, left, right)
    }

    #[cfg(test)]
    pub fn less(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::LessThan, left, right)
    }

    #[cfg(test)]
    pub fn plus(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::Plus, left, right)
    }

    #[cfg(test)]
    pub fn minus(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::Minus, left, right)
    }
}

impl<T: BufRead> Parser<T> {
    pub fn demand_expression(&mut self) -> Result<Expression> {
        match self.try_parse_expression()? {
            Some(e) => Ok(e),
            None => Err(format!(
                "Expected expression, found {:?}",
                self.buf_lexer.read()?
            )),
        }
    }

    pub fn try_parse_expression(&mut self) -> Result<Option<Expression>> {
        let left_side = self._try_parse_single_expression()?;
        match left_side {
            None => Ok(None),
            Some(exp) => Ok(Some(self._try_parse_second_expression(exp)?)),
        }
    }

    fn _try_parse_single_expression(&mut self) -> Result<Option<Expression>> {
        let next = self.buf_lexer.read()?;
        match next {
            Lexeme::Symbol('"') => Ok(Some(self._parse_string_literal()?)),
            Lexeme::Word(word) => Ok(Some(self._parse_word(word)?)),
            Lexeme::Digits(digits) => Ok(Some(self._parse_number_literal(digits)?)),
            Lexeme::Symbol('.') => Ok(Some(self._parse_floating_point_literal(0)?)),
            Lexeme::Symbol('-') => {
                self.buf_lexer.consume();
                match self._try_parse_single_expression()? {
                    Some(e) => Ok(Some(Expression::unary_minus(e))),
                    None => self
                        .buf_lexer
                        .err("Expected expression after minus operator"),
                }
            }
            _ => Ok(None),
        }
    }

    fn _try_parse_second_expression(&mut self, left_side: Expression) -> Result<Expression> {
        let operand = self._try_parse_operand()?;
        match operand {
            Some(op) => {
                self.buf_lexer.skip_whitespace()?;
                let right_side = self.demand_expression()?;
                Ok(Expression::BinaryExpression(
                    op,
                    Box::new(left_side),
                    Box::new(right_side),
                ))
            }
            None => Ok(left_side),
        }
    }

    /// Parses a comma separated list of expressions.
    pub fn parse_expression_list(&mut self) -> Result<Vec<Expression>> {
        let mut args: Vec<Expression> = vec![];
        let optional_first_arg = self.try_parse_expression()?;
        if let Some(first_arg) = optional_first_arg {
            args.push(first_arg);
            while self._read_comma_between_arguments()? {
                self.buf_lexer.skip_whitespace()?;
                let next_arg = self.demand_expression()?;
                args.push(next_arg);
            }
        }

        Ok(args)
    }

    fn _read_comma_between_arguments(&mut self) -> Result<bool> {
        // skip whitespace after previous arg
        self.buf_lexer.skip_whitespace()?;
        self.buf_lexer.try_consume_symbol(',')
    }

    fn _parse_string_literal(&mut self) -> Result<Expression> {
        // verify we read a double quote
        self.buf_lexer.demand_symbol('"')?;

        let mut buf: String = String::new();

        // read until we hit the next double quote
        loop {
            let l = self.buf_lexer.read()?;
            self.buf_lexer.consume();
            match l {
                Lexeme::Symbol('"') => break,
                Lexeme::EOF => return self.buf_lexer.err("EOF while looking for end of string"),
                Lexeme::EOL(_) => {
                    return self
                        .buf_lexer
                        .err("Unexpected new line while looking for end of string")
                }
                _ => l.push_to(&mut buf),
            }
        }

        Ok(Expression::StringLiteral(buf))
    }

    fn _parse_number_literal(&mut self, digits: u32) -> Result<Expression> {
        // consume digits
        self.buf_lexer.consume();
        let next = self.buf_lexer.read()?;
        match next {
            Lexeme::Symbol('.') => self._parse_floating_point_literal(digits),
            _ => {
                // no decimal point, just integer
                match i32::try_from(digits) {
                    Ok(i) => Ok(Expression::IntegerLiteral(i)),
                    Err(err) => self
                        .buf_lexer
                        .err(format!("Could not convert digits to i32: {}", err)),
                }
            }
        }
    }

    fn _demand_digits(&mut self) -> Result<u32> {
        let next = self.buf_lexer.read()?;
        match next {
            Lexeme::Digits(digits) => {
                self.buf_lexer.consume();
                Ok(digits)
            }
            _ => self.buf_lexer.err("Expected digits"),
        }
    }

    fn _parse_floating_point_literal(&mut self, integer_digits: u32) -> Result<Expression> {
        // consume dot
        self.buf_lexer.consume();
        let fraction_digits = self._demand_digits()?;
        let is_double = self.buf_lexer.try_consume_symbol('#')?;
        if is_double {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f64>() {
                Ok(f) => Ok(Expression::DoubleLiteral(f)),
                Err(err) => self
                    .buf_lexer
                    .err(format!("Could not convert digits to f64: {}", err)),
            }
        } else {
            match format!("{}.{}", integer_digits, fraction_digits).parse::<f32>() {
                Ok(f) => Ok(Expression::SingleLiteral(f)),
                Err(err) => self
                    .buf_lexer
                    .err(format!("Could not convert digits to f32: {}", err)),
            }
        }
    }

    fn _parse_word(&mut self, word: String) -> Result<Expression> {
        self.buf_lexer.consume();
        // is it maybe a qualified variable name
        let qualifier = self.try_parse_type_qualifier()?;
        // it could be a function call?
        if self.buf_lexer.try_consume_symbol('(')? {
            let args = self.parse_expression_list()?;
            self.buf_lexer.demand_symbol(')')?;
            Ok(Expression::FunctionCall(QName::new(word, qualifier), args))
        } else {
            Ok(Expression::VariableName(QName::new(word, qualifier)))
        }
    }

    fn _try_parse_operand(&mut self) -> Result<Option<Operand>> {
        self.buf_lexer.mark();
        self.buf_lexer.skip_whitespace()?;
        if self.buf_lexer.try_consume_symbol('<')? {
            self._less_or_lte()
        } else if self.buf_lexer.try_consume_symbol('+')? {
            Ok(Some(Operand::Plus))
        } else if self.buf_lexer.try_consume_symbol('-')? {
            Ok(Some(Operand::Minus))
        } else {
            self.buf_lexer.backtrack();
            Ok(None)
        }
    }

    fn _less_or_lte(&mut self) -> Result<Option<Operand>> {
        if self.buf_lexer.try_consume_symbol('=')? {
            Ok(Some(Operand::LessOrEqualThan))
        } else {
            Ok(Some(Operand::LessThan))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn assert_parse_literal<TResult>(input: &str, expected: TResult)
    where
        TResult: std::fmt::Display + PartialEq,
        Expression: From<TResult>,
    {
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
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
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::VariableName(QName::from_str("A").unwrap())
        );
    }

    #[test]
    fn test_function_call_expression_no_args() {
        let input = "IsValid()";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(QName::Untyped("IsValid".to_string()), vec![])
        );
    }

    #[test]
    fn test_function_call_qualified_expression_no_args() {
        let input = "IsValid%()";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                QName::Typed(QualifiedName::new(
                    "IsValid".to_string(),
                    TypeQualifier::PercentInteger
                )),
                vec![]
            )
        );
    }

    #[test]
    fn test_function_call_expression_one_arg() {
        let input = "IsValid(42)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                QName::Untyped("IsValid".to_string()),
                vec![Expression::IntegerLiteral(42)]
            )
        );
    }

    #[test]
    fn test_function_call_expression_two_args() {
        let input = "CheckProperty(42, \"age\")";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                QName::Untyped("CheckProperty".to_string()),
                vec![Expression::from(42), Expression::from("age")]
            )
        );
    }

    #[test]
    fn test_function_call_in_function_call() {
        let input = "CheckProperty(LookupName(\"age\"), Confirm())";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::FunctionCall(
                QName::Untyped("CheckProperty".to_string()),
                vec![
                    Expression::FunctionCall(
                        QName::Untyped("LookupName".to_string()),
                        vec![Expression::from("age")]
                    ),
                    Expression::FunctionCall(QName::Untyped("Confirm".to_string()), vec![])
                ]
            )
        );
    }

    #[test]
    fn test_lte() {
        let input = "N <= 1";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::LessOrEqualThan,
                Box::new(Expression::variable_name_unqualified("N")),
                Box::new(Expression::IntegerLiteral(1))
            )
        );
    }

    #[test]
    fn test_less_than() {
        let input = "A < B";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::LessThan,
                Box::new(Expression::variable_name_unqualified("A")),
                Box::new(Expression::variable_name_unqualified("B"))
            )
        );
    }

    #[test]
    fn test_plus() {
        let input = "N + 1";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Plus,
                Box::new(Expression::variable_name_unqualified("N")),
                Box::new(Expression::IntegerLiteral(1))
            )
        );
    }

    #[test]
    fn test_minus() {
        let input = "N - 2";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Minus,
                Box::new(Expression::variable_name_unqualified("N")),
                Box::new(Expression::IntegerLiteral(2))
            )
        );
    }

    #[test]
    fn test_negated_variable() {
        let input = "-N";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::unary_minus(Expression::variable_name_unqualified("N"))
        );
    }

    #[test]
    fn test_fib_expression() {
        let input = "Fib(N - 1) + Fib(N - 2)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::BinaryExpression(
                Operand::Plus,
                Box::new(Expression::FunctionCall(
                    QName::Untyped("Fib".to_string()),
                    vec![Expression::BinaryExpression(
                        Operand::Minus,
                        Box::new(Expression::variable_name_unqualified("N")),
                        Box::new(Expression::IntegerLiteral(1))
                    )]
                )),
                Box::new(Expression::FunctionCall(
                    QName::Untyped("Fib".to_string()),
                    vec![Expression::minus(
                        Expression::variable_name_unqualified("N"),
                        Expression::IntegerLiteral(2)
                    )]
                ))
            )
        );
    }

    #[test]
    fn test_negated_function_call() {
        let input = "-Fib(-N)";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(
            expression,
            Expression::unary_minus(Expression::FunctionCall(
                QName::Untyped("Fib".to_string()),
                vec![Expression::unary_minus(
                    Expression::variable_name_unqualified("N")
                )]
            ))
        );
    }
}
