use crate::built_ins::BuiltInSub;
use crate::common::{FileHandle, Locatable};
use crate::linter::types::{DimName, DimNameNode, ExpressionNode};
use crate::parser::{BareName, Operator, QualifiedNameNode};
use crate::variant::Variant;

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: DimName,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<DimNameNode>,
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
    Is(Operator, ExpressionNode),
    Range(ExpressionNode, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(DimName, ExpressionNode),
    Const(QualifiedNameNode, Variant),
    SubCall(BareName, Vec<ExpressionNode>),
    BuiltInSubCall(BuiltInSub, Vec<ExpressionNode>),

    IfBlock(IfBlockNode),
    SelectCase(SelectCaseNode),

    ForLoop(ForLoopNode),
    While(ConditionalBlockNode),

    ErrorHandler(BareName),
    Label(BareName),
    GoTo(BareName),

    Comment(String),
    Dim(DimNameNode),
    Print(PrintNode),
}

pub type StatementNode = Locatable<Statement>;
pub type StatementNodes = Vec<StatementNode>;

#[derive(Clone, Debug, PartialEq)]
pub struct PrintNode {
    pub file_number: Option<FileHandle>,
    pub lpt1: bool,
    pub format_string: Option<ExpressionNode>,
    pub args: Vec<PrintArg>,
}

impl PrintNode {
    pub fn one(e: ExpressionNode) -> Self {
        Self {
            file_number: None,
            lpt1: false,
            format_string: None,
            args: vec![PrintArg::Expression(e)],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PrintArg {
    Comma,
    Semicolon,
    Expression(ExpressionNode),
}