use super::error::*;
use crate::common::*;
use crate::parser::*;

pub type QNameNode = Locatable<QualifiedName>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    #[allow(dead_code)]
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(QualifiedName),
    FunctionCall(QualifiedName, Vec<ExpressionNode>),
    BinaryExpression(Operand, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperand, Box<ExpressionNode>),
}

impl Expression {
    pub fn try_qualifier(&self) -> Result<TypeQualifier, Error> {
        match self {
            Self::SingleLiteral(_) => Ok(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => Ok(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => Ok(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => Ok(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => Ok(TypeQualifier::AmpersandLong),
            Self::Variable(name) | Self::Constant(name) | Self::FunctionCall(name, _) => {
                Ok(name.qualifier())
            }
            Self::BinaryExpression(op, l, r) => {
                let q_left = l.as_ref().as_ref().try_qualifier()?;
                let q_right = r.as_ref().as_ref().try_qualifier()?;
                if q_left.can_cast_to(q_right) {
                    match op {
                        Operand::Plus | Operand::Minus => Ok(q_left),
                        Operand::LessThan | Operand::LessOrEqualThan => {
                            Ok(TypeQualifier::PercentInteger)
                        }
                    }
                } else {
                    err(LinterError::TypeMismatch, r.as_ref().location())
                }
            }
            Self::UnaryExpression(_, c) => {
                let q_child = c.as_ref().as_ref().try_qualifier()?;
                if q_child == TypeQualifier::DollarString {
                    // no unary operator currently applicable to strings
                    err(LinterError::TypeMismatch, c.as_ref().location())
                } else {
                    Ok(q_child)
                }
            }
        }
    }
}

pub type ExpressionNode = Locatable<Expression>;

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: QNameNode,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<QNameNode>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBlockNode {
    pub condition: ExpressionNode,
    pub statements: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlockNode {
    pub if_block: ConditionalBlockNode,
    pub else_if_blocks: Vec<ConditionalBlockNode>,
    pub else_block: Option<StatementNodes>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    SubCall(BareName, Vec<ExpressionNode>),
    ForLoop(ForLoopNode),
    IfBlock(IfBlockNode),
    Assignment(QualifiedName, ExpressionNode),
    While(ConditionalBlockNode),
    Const(QNameNode, ExpressionNode),
    ErrorHandler(CaseInsensitiveString),
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),
    SetReturnValue(ExpressionNode),
}

pub type StatementNode = Locatable<Statement>;
pub type StatementNodes = Vec<StatementNode>;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionImplementation {
    pub name: QNameNode,
    pub params: Vec<QNameNode>,
    pub body: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubImplementation {
    pub name: BareNameNode,
    pub params: Vec<QNameNode>,
    pub body: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum TopLevelToken {
    /// A function implementation
    FunctionImplementation(FunctionImplementation),

    /// A simple or compound statement
    Statement(Statement),

    /// A sub implementation
    SubImplementation(SubImplementation),
}

pub type TopLevelTokenNode = Locatable<TopLevelToken>;
pub type ProgramNode = Vec<TopLevelTokenNode>;
