use super::{ExpressionType, HasExpressionType};
use crate::built_ins::BuiltInFunction;
use crate::common::{CanCastTo, Locatable, QError, QErrorNode, ToLocatableError};
use crate::parser::{BareName, Name, Operator, QualifiedName, TypeQualifier, UnaryOperator};

#[cfg(test)]
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(Name, ExpressionType),
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
    Property(
        // the left side of the property, before the dot
        Box<Expression>,
        // the property name (converted to BareName)
        BareName,
        // the resolved type of the property
        ExpressionType,
    ),
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
        let q_name = QualifiedName::try_from(name).unwrap();
        Expression::from(q_name)
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Expression::Variable(name.into(), ExpressionType::UserDefined(type_name.into()))
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
            Self::Variable(_, expression_type)
            | Self::Property(_, _, expression_type)
            | Self::BinaryExpression(_, _, _, expression_type)
            | Self::ArrayElement(_, _, expression_type) => expression_type.clone(),
            Self::Constant(QualifiedName { qualifier, .. })
            | Self::FunctionCall(QualifiedName { qualifier, .. }, _) => {
                ExpressionType::BuiltIn(*qualifier)
            }
            Self::BuiltInFunctionCall(f, _) => ExpressionType::BuiltIn(f.into()),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.as_ref().expression_type(),
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

impl From<QualifiedName> for Expression {
    fn from(var_name: QualifiedName) -> Self {
        let q = var_name.qualifier;
        Self::Variable(var_name.into(), ExpressionType::BuiltIn(q))
    }
}
