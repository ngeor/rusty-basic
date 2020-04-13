use super::NameNode;
use crate::common::{HasLocation, Locatable, Location};

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

pub type OperandNode = Locatable<Operand>;

pub type UnaryOperandNode = Locatable<UnaryOperand>;

#[derive(Clone, Debug, PartialEq)]
pub enum ExpressionNode {
    SingleLiteral(f32, Location),
    DoubleLiteral(f64, Location),
    StringLiteral(String, Location),
    IntegerLiteral(i32, Location),
    LongLiteral(i64, Location),
    VariableName(NameNode),
    FunctionCall(NameNode, Vec<ExpressionNode>),
    BinaryExpression(OperandNode, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperandNode, Box<ExpressionNode>),
}

impl ExpressionNode {
    pub fn unary(operand: UnaryOperandNode, child: ExpressionNode) -> ExpressionNode {
        ExpressionNode::UnaryExpression(operand, Box::new(child))
    }

    pub fn unary_minus(operand: UnaryOperandNode, child: ExpressionNode) -> ExpressionNode {
        match child {
            ExpressionNode::SingleLiteral(n, _) => {
                ExpressionNode::SingleLiteral(-n, operand.location())
            }
            ExpressionNode::DoubleLiteral(n, _) => {
                ExpressionNode::DoubleLiteral(-n, operand.location())
            }
            ExpressionNode::IntegerLiteral(n, _) => {
                ExpressionNode::IntegerLiteral(-n, operand.location())
            }
            ExpressionNode::LongLiteral(n, _) => {
                ExpressionNode::LongLiteral(-n, operand.location())
            }
            _ => ExpressionNode::unary(operand, child),
        }
    }
}

impl HasLocation for ExpressionNode {
    fn location(&self) -> Location {
        match self {
            ExpressionNode::VariableName(n) => n.location(),
            ExpressionNode::SingleLiteral(_, pos)
            | ExpressionNode::DoubleLiteral(_, pos)
            | ExpressionNode::StringLiteral(_, pos)
            | ExpressionNode::IntegerLiteral(_, pos)
            | ExpressionNode::LongLiteral(_, pos) => *pos,
            ExpressionNode::FunctionCall(n, _) => n.location(),
            ExpressionNode::BinaryExpression(_, left, _) => left.location(),
            ExpressionNode::UnaryExpression(op, _) => op.location(), // because op is probably left of child
        }
    }
}

#[cfg(test)]
impl PartialEq<i32> for ExpressionNode {
    fn eq(&self, other: &i32) -> bool {
        match self {
            ExpressionNode::IntegerLiteral(i, _) => i == other,
            _ => false,
        }
    }
}
