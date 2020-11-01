use super::{DimName, ExpressionType, HasExpressionType};
use crate::built_ins::BuiltInFunction;
use crate::common::{CanCastTo, Locatable, QError, QErrorNode, ToLocatableError};
use crate::parser::{Name, Operator, QualifiedName, TypeQualifier, UnaryOperator};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(DimName),
    FunctionCall(QualifiedName, Vec<ExpressionNode>),
    ArrayElement(
        // the name of the array (unqualified only for user defined types)
        Name,
        // the array indices
        Vec<ExpressionNode>,
        // the type of the elements
        ExpressionType,
    ),
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(
        Operator,
        Box<ExpressionNode>,
        Box<ExpressionNode>,
        // the resolved type definition (e.g. 1 + 2.1 -> single)
        ExpressionType,
    ),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
}

pub type ExpressionNode = Locatable<Expression>;
pub type ExpressionNodes = Vec<ExpressionNode>;

impl Expression {
    pub fn binary(
        left: ExpressionNode,
        right: ExpressionNode,
        op: Operator,
    ) -> Result<Self, QErrorNode> {
        // get the types
        let t_left = left.expression_type();
        let t_right = right.expression_type();
        // get the cast type
        match t_left.cast_binary_op(t_right, op) {
            Some(type_definition) => Ok(Expression::BinaryExpression(
                op,
                Box::new(left),
                Box::new(right),
                type_definition,
            )),
            None => Err(QError::TypeMismatch).with_err_at(&right),
        }
    }

    #[cfg(test)]
    pub fn var(name: &str) -> Self {
        Expression::Variable(DimName::parse(name))
    }
}

impl HasExpressionType for Expression {
    fn expression_type(&self) -> ExpressionType {
        match self {
            Self::SingleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => ExpressionType::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Variable(name) => name.expression_type(),
            Self::Constant(QualifiedName { qualifier, .. })
            | Self::FunctionCall(QualifiedName { qualifier, .. }, _) => {
                ExpressionType::BuiltIn(*qualifier)
            }
            Self::BuiltInFunctionCall(f, _) => ExpressionType::BuiltIn(f.into()),
            Self::BinaryExpression(_, _, _, type_definition) => type_definition.clone(),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.as_ref().expression_type(),
            Self::ArrayElement(_, _, element_type) => element_type.clone(),
        }
    }
}

impl CanCastTo<TypeQualifier> for Expression {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.expression_type().can_cast_to(other)
    }
}

impl<T: HasExpressionType> CanCastTo<&T> for Expression {
    fn can_cast_to(&self, other: &T) -> bool {
        let other_type_definition = other.expression_type();
        self.expression_type().can_cast_to(&other_type_definition)
    }
}
