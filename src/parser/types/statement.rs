use super::{Expression, Name};
use crate::common::CaseInsensitiveString;

pub type Block = Vec<Statement>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    SubCall(CaseInsensitiveString, Vec<Expression>),
    ForLoop(ForLoop),
    IfBlock(IfBlock),
    Assignment(Name, Expression),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    pub variable_name: Name,
    pub lower_bound: Expression,
    pub upper_bound: Expression,
    pub step: Option<Expression>,
    pub statements: Block,
    pub next_counter: Option<Name>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBlock {
    pub condition: Expression,
    pub block: Block,
}

#[cfg(test)]
impl ConditionalBlock {
    pub fn new(condition: Expression, block: Block) -> ConditionalBlock {
        ConditionalBlock { condition, block }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub if_block: ConditionalBlock,
    pub else_if_blocks: Vec<ConditionalBlock>,
    pub else_block: Option<Block>,
}
