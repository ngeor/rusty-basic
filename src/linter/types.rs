use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::*;
use crate::parser::*;

pub type QNameNode = Locatable<QualifiedName>;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(QualifiedName),
    FunctionCall(QualifiedName, Vec<ExpressionNode>),
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(Operand, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperand, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
    FileHandle(FileHandle),
}

impl Expression {
    pub fn try_qualifier(&self, pos: Location) -> Result<TypeQualifier, QErrorNode> {
        match self {
            Self::SingleLiteral(_) => Ok(TypeQualifier::BangSingle),
            Self::DoubleLiteral(_) => Ok(TypeQualifier::HashDouble),
            Self::StringLiteral(_) => Ok(TypeQualifier::DollarString),
            Self::IntegerLiteral(_) => Ok(TypeQualifier::PercentInteger),
            Self::LongLiteral(_) => Ok(TypeQualifier::AmpersandLong),
            Self::Variable(name) | Self::Constant(name) | Self::FunctionCall(name, _) => {
                Ok(name.qualifier())
            }
            Self::BuiltInFunctionCall(f, _) => Ok(f.qualifier()),
            Self::BinaryExpression(op, l, r) => {
                let q_left = l.as_ref().try_qualifier()?;
                let q_right = r.as_ref().try_qualifier()?;
                super::operand_type::cast_binary_op(*op, q_left, q_right)
                    .ok_or_else(|| QError::TypeMismatch)
                    .with_err_at(r.pos())
            }
            Self::UnaryExpression(op, c) => {
                let q_child = c.as_ref().try_qualifier()?;
                super::operand_type::cast_unary_op(*op, q_child)
                    .ok_or_else(|| QError::TypeMismatch)
                    .with_err_at(c.as_ref())
            }
            Self::Parenthesis(c) => c.as_ref().try_qualifier(),
            Self::FileHandle(_) => Err(QError::TypeMismatch).with_err_at(pos),
        }
    }
}

pub type ExpressionNode = Locatable<Expression>;

impl ExpressionNode {
    pub fn try_qualifier(&self) -> Result<TypeQualifier, QErrorNode> {
        self.as_ref().try_qualifier(self.pos())
    }
}

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
pub struct SelectCaseNode {
    /// The expression been matched
    pub expr: ExpressionNode,
    /// The case statements
    pub case_blocks: Vec<CaseBlockNode>,
    /// An optional CASE ELSE block
    pub else_block: Option<StatementNodes>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseBlockNode {
    pub expr: CaseExpression,
    pub statements: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaseExpression {
    Simple(ExpressionNode),
    Is(Operand, ExpressionNode),
    Range(ExpressionNode, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(QualifiedName, ExpressionNode),
    Const(QNameNode, ExpressionNode),
    SubCall(BareName, Vec<ExpressionNode>),
    BuiltInSubCall(BuiltInSub, Vec<ExpressionNode>),

    IfBlock(IfBlockNode),
    SelectCase(SelectCaseNode),

    ForLoop(ForLoopNode),
    While(ConditionalBlockNode),

    ErrorHandler(CaseInsensitiveString),
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),

    SetReturnValue(ExpressionNode),
    Comment(String),
    Dim(DeclaredNameNode),
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
