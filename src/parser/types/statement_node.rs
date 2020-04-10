use super::{ConditionalBlock, ExpressionNode, ForLoop, IfBlock, NameNode, Statement};
use crate::common::{Location, StripLocation};

pub type BlockNode = Vec<StatementNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum StatementNode {
    SubCall(NameNode, Vec<ExpressionNode>),
    ForLoop(ForLoopNode),
    IfBlock(IfBlockNode),
    Assignment(NameNode, ExpressionNode),
    Whitespace(String),
}

impl StripLocation<Statement> for StatementNode {
    fn strip_location(&self) -> Statement {
        match self {
            StatementNode::SubCall(n, args) => {
                Statement::SubCall(n.name().clone(), args.strip_location())
            }
            StatementNode::ForLoop(f) => Statement::ForLoop(f.strip_location()),
            StatementNode::IfBlock(i) => Statement::IfBlock(i.strip_location()),
            StatementNode::Assignment(left, right) => {
                Statement::Assignment(left.strip_location(), right.strip_location())
            }
            StatementNode::Whitespace(x) => Statement::Whitespace(x.clone()),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: NameNode,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: BlockNode,
    pub next_counter: Option<NameNode>,
    pub pos: Location,
}

impl StripLocation<ForLoop> for ForLoopNode {
    fn strip_location(&self) -> ForLoop {
        ForLoop {
            variable_name: self.variable_name.strip_location(),
            lower_bound: self.lower_bound.strip_location(),
            upper_bound: self.upper_bound.strip_location(),
            step: self.step.strip_location(),
            statements: self.statements.strip_location(),
            next_counter: self.next_counter.strip_location(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBlockNode {
    pub condition: ExpressionNode,
    pub block: BlockNode,
    pub pos: Location,
}

impl StripLocation<ConditionalBlock> for ConditionalBlockNode {
    fn strip_location(&self) -> ConditionalBlock {
        ConditionalBlock {
            condition: self.condition.strip_location(),
            block: self.block.strip_location(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlockNode {
    pub if_block: ConditionalBlockNode,
    pub else_if_blocks: Vec<ConditionalBlockNode>,
    pub else_block: Option<BlockNode>,
}

impl StripLocation<IfBlock> for IfBlockNode {
    fn strip_location(&self) -> IfBlock {
        IfBlock {
            if_block: self.if_block.strip_location(),
            else_if_blocks: self.else_if_blocks.strip_location(),
            else_block: self.else_block.strip_location(),
        }
    }
}
