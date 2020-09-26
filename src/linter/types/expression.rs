use super::{DimName, ExpressionType, HasExpressionType};
use crate::built_ins::BuiltInFunction;
use crate::common::{CanCastTo, FileHandle, Locatable};
use crate::parser::{Operator, QualifiedName, TypeQualifier, UnaryOperator};

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
    FileHandle(FileHandle),
}

pub type ExpressionNode = Locatable<Expression>;

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
            Self::FileHandle(_) => ExpressionType::FileHandle,
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
