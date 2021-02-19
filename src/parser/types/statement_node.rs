use super::{
    BareName, DimNameNode, Expression, ExpressionNode, ExpressionNodes, NameNode, Operator,
    PrintNode,
};
use crate::built_ins::BuiltInSub;
use crate::common::*;

pub type StatementNode = Locatable<Statement>;
pub type StatementNodes = Vec<StatementNode>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    // A = 42
    // A.Hello = 42 at the parser state it is not known if this is a member variable or not
    // A$ = "hello"
    Assignment(Expression, ExpressionNode),

    Const(NameNode, ExpressionNode),

    /// DIM A (Bare)
    /// DIM A$ (Compact)
    /// DIM A AS INTEGER (ExtendedBuiltIn)
    /// DIM A AS STRING (without length)
    /// DIM A AS STRING * 4 (with fixed length)
    /// DIM A AS UserDefinedType
    Dim(DimNameNode),

    SubCall(BareName, ExpressionNodes),
    BuiltInSubCall(BuiltInSub, ExpressionNodes),
    IfBlock(IfBlockNode),
    SelectCase(SelectCaseNode),

    ForLoop(ForLoopNode),
    While(ConditionalBlockNode),

    ErrorHandler(CaseInsensitiveString),
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),

    GoSub(CaseInsensitiveString),
    Return(Option<CaseInsensitiveString>),

    Exit(ExitObject),

    Comment(String),

    // some built-ins have special syntax
    Print(PrintNode),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitObject {
    Function,
    Sub,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: ExpressionNode,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<ExpressionNode>,
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
    /// Holds an optional inline comment after SELECT CASE X e.g. SELECT CASE X ' make a choice
    pub inline_comments: Vec<Locatable<String>>,
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
