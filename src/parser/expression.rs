use super::{Parser, QName, TypeQualifier};
use crate::common::Result;
use crate::lexer::Lexeme;
use std::convert::TryFrom;
use std::io::BufRead;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Operand {
    LessOrEqualThan,
    LessThan,
    Plus,
    Minus,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    StringLiteral(String),
    VariableName(QName),
    IntegerLiteral(i32),
    FunctionCall(QName, Vec<Expression>),
    BinaryExpression(Operand, Box<Expression>, Box<Expression>),
}

impl Expression {
    /// Creates a new IntegerLiteral expression
    pub fn integer_literal(value: i32) -> Expression {
        Expression::IntegerLiteral(value)
    }

    /// Creates a new VariableName expression with a qualified type
    pub fn variable_name_qualified<S: AsRef<str>>(
        name: S,
        type_qualifier: TypeQualifier,
    ) -> Expression {
        Expression::VariableName(QName::Typed(name.as_ref().to_string(), type_qualifier))
    }

    /// Creates a new VariableName expression without a qualified type
    pub fn variable_name_unqualified<S: AsRef<str>>(name: S) -> Expression {
        Expression::VariableName(QName::Untyped(name.as_ref().to_string()))
    }

    /// Creates a new StringLiteral expression
    pub fn string_literal<S: AsRef<str>>(literal: S) -> Expression {
        Expression::StringLiteral(literal.as_ref().to_string())
    }

    pub fn binary(operand: Operand, left: Expression, right: Expression) -> Expression {
        Expression::BinaryExpression(operand, Box::new(left), Box::new(right))
    }

    pub fn lte(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::LessOrEqualThan, left, right)
    }

    pub fn plus(left: Expression, right: Expression) -> Expression {
        Expression::binary(Operand::Plus, left, right)
    }

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
        self.buf_lexer.consume();
        match i32::try_from(digits) {
            Ok(i) => Ok(Expression::IntegerLiteral(i)),
            Err(err) => self
                .buf_lexer
                .err(format!("Could not convert digits to i32: {}", err)),
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

    #[test]
    fn test_string_literal_expression() {
        let input = "\"hello, world\"";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(expression, Expression::string_literal("hello, world"));
    }

    #[test]
    fn test_numeric_expression() {
        let input = "42";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(expression, Expression::IntegerLiteral(42));
    }

    #[test]
    fn test_variable_expression() {
        let input = "A";
        let mut parser = Parser::from(input);
        let expression = parser.demand_expression().unwrap();
        assert_eq!(expression, Expression::variable_name_unqualified("A"));
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
                QName::Typed("IsValid".to_string(), TypeQualifier::PercentInteger),
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
                vec![
                    Expression::IntegerLiteral(42),
                    Expression::string_literal("age")
                ]
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
                        vec![Expression::string_literal("age")]
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
}
