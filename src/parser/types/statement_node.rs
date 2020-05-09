use super::{ArgumentNodes, BareName, ExpressionNode, Name, NameNode};
use crate::common::*;

pub type StatementNodes = Vec<StatementNode>;
pub type StatementNode = Locatable<Statement>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    SubCall(BareName, ArgumentNodes),
    ForLoop(ForLoopNode),
    IfBlock(IfBlockNode),
    Assignment(Name, ExpressionNode),
    While(ConditionalBlockNode),
    Const(NameNode, ExpressionNode),
    ErrorHandler(CaseInsensitiveString),
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: NameNode,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<NameNode>,
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
