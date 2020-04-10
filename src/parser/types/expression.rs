use super::Name;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Operand {
    LessOrEqualThan,
    LessThan,
    Plus,
    Minus,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum UnaryOperand {
    // Plus,
    Minus,
    // Not,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    VariableName(Name),
    FunctionCall(Name, Vec<Expression>),
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
    pub fn variable_name(name: &str) -> Expression {
        Expression::VariableName(Name::from(name))
    }

    #[cfg(test)]
    pub fn binary(operand: Operand, left: Expression, right: Expression) -> Expression {
        Expression::BinaryExpression(operand, Box::new(left), Box::new(right))
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
