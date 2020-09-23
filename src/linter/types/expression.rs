use super::{DimName, HasTypeDefinition, TypeDefinition};
use crate::built_ins::BuiltInFunction;
use crate::common::{CanCastTo, FileHandle, Locatable};
use crate::parser::{HasQualifier, Operator, QualifiedName, TypeQualifier, UnaryOperator};

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
        TypeDefinition,
    ),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
    FileHandle(FileHandle),
}

pub type ExpressionNode = Locatable<Expression>;

impl HasTypeDefinition for Expression {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::SingleLiteral(_) => TypeDefinition::BuiltIn(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => TypeDefinition::BuiltIn(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => TypeDefinition::BuiltIn(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => TypeDefinition::BuiltIn(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => TypeDefinition::BuiltIn(TypeQualifier::AmpersandLong),
            Self::Variable(name) => name.type_definition(),
            Self::Constant(name) | Self::FunctionCall(name, _) => {
                TypeDefinition::BuiltIn(name.qualifier())
            }
            Self::BuiltInFunctionCall(f, _) => TypeDefinition::BuiltIn(f.qualifier()),
            Self::BinaryExpression(_, _, _, type_definition) => type_definition.clone(),
            Self::UnaryExpression(_, c) | Self::Parenthesis(c) => c.as_ref().type_definition(),
            Self::FileHandle(_) => TypeDefinition::FileHandle,
        }
    }
}

impl CanCastTo<TypeQualifier> for Expression {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.type_definition().can_cast_to(other)
    }
}

impl<T: HasTypeDefinition> CanCastTo<&T> for Expression {
    fn can_cast_to(&self, other: &T) -> bool {
        let other_type_definition = other.type_definition();
        self.type_definition().can_cast_to(&other_type_definition)
    }
}
