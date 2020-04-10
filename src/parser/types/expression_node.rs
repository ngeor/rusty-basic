use super::{Expression, NameNode, Operand, UnaryOperand};
use crate::common::{
    AddLocation, HasLocation, Locatable, Location, StripLocation, StripLocationRef,
};

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

impl StripLocation<Expression> for ExpressionNode {
    fn strip_location(&self) -> Expression {
        match self {
            ExpressionNode::SingleLiteral(x, _) => Expression::SingleLiteral(*x),
            ExpressionNode::DoubleLiteral(x, _) => Expression::DoubleLiteral(*x),
            ExpressionNode::StringLiteral(x, _) => Expression::StringLiteral(x.clone()),
            ExpressionNode::IntegerLiteral(x, _) => Expression::IntegerLiteral(*x),
            ExpressionNode::LongLiteral(x, _) => Expression::LongLiteral(*x),
            ExpressionNode::VariableName(n) => Expression::VariableName(n.strip_location()),
            ExpressionNode::FunctionCall(n, args) => {
                Expression::FunctionCall(n.strip_location(), args.strip_location())
            }
            ExpressionNode::BinaryExpression(op, left, right) => Expression::BinaryExpression(
                *op.strip_location_ref(),
                left.strip_location(),
                right.strip_location(),
            ),
            ExpressionNode::UnaryExpression(op, child) => {
                Expression::UnaryExpression(*op.strip_location_ref(), child.strip_location())
            }
        }
    }
}

impl AddLocation<ExpressionNode> for Expression {
    fn add_location(&self, pos: Location) -> ExpressionNode {
        match self {
            Expression::SingleLiteral(x) => ExpressionNode::SingleLiteral(*x, pos),
            Expression::DoubleLiteral(x) => ExpressionNode::DoubleLiteral(*x, pos),
            Expression::StringLiteral(x) => ExpressionNode::StringLiteral(x.clone(), pos),
            Expression::IntegerLiteral(x) => ExpressionNode::IntegerLiteral(*x, pos),
            Expression::LongLiteral(x) => ExpressionNode::LongLiteral(*x, pos),
            Expression::VariableName(n) => ExpressionNode::VariableName(n.add_location(pos)),
            Expression::FunctionCall(n, args) => ExpressionNode::FunctionCall(
                n.add_location(pos),
                args.iter().map(|x| x.add_location(pos)).collect(),
            ),
            Expression::BinaryExpression(op, left, right) => ExpressionNode::BinaryExpression(
                op.add_location(pos),
                left.add_location(pos),
                right.add_location(pos),
            ),
            Expression::UnaryExpression(op, child) => {
                ExpressionNode::UnaryExpression(op.add_location(pos), child.add_location(pos))
            }
        }
    }
}
