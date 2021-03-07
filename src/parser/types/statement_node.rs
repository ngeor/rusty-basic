use super::{
    BareName, DimNameNodes, Expression, ExpressionNode, ExpressionNodes, NameNode, Operator,
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
    /// DIM SHARED A
    /// DIM A(1 TO 2)
    /// DIM A, B, C
    Dim(DimList),

    SubCall(BareName, ExpressionNodes),
    BuiltInSubCall(BuiltInSub, ExpressionNodes),

    IfBlock(IfBlockNode),
    SelectCase(SelectCaseNode),

    ForLoop(ForLoopNode),
    While(ConditionalBlockNode),
    DoLoop(DoLoopNode),

    OnError(OnErrorOption),
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),
    GoSub(CaseInsensitiveString),
    Resume(ResumeOption),
    Return(Option<CaseInsensitiveString>),
    Exit(ExitObject),

    Comment(String),

    Print(PrintNode),

    End,
    System,
}

/// A list of variables defined in a DIM statement.
#[derive(Clone, Debug, PartialEq)]
pub struct DimList {
    /// Specifies if the variables are shared. Can only be used on the global
    /// module. If shared, the variables are available in functions/subs.
    pub shared: bool,

    /// The variables defined in the DIM statement.
    pub variables: DimNameNodes,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitObject {
    Function,
    Sub,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResumeOption {
    Bare,
    Next,
    Label(CaseInsensitiveString),
}

#[derive(Clone, Debug, PartialEq)]
pub enum OnErrorOption {
    Next,
    Label(CaseInsensitiveString),
    Zero,
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
    pub expression_list: Vec<CaseExpression>,
    pub statements: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaseExpression {
    Simple(ExpressionNode),
    Is(Operator, ExpressionNode),
    Range(ExpressionNode, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoLoopNode {
    pub condition: ExpressionNode,
    pub statements: StatementNodes,
    pub position: DoLoopConditionPosition,
    pub kind: DoLoopConditionKind,
}

/// Indicates where the condition expression of
/// the DO LOOP is located.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoLoopConditionPosition {
    /// The condition is placed after the DO keyword, e.g.
    /// DO WHILE A > 0 ... LOOP
    Top,

    /// The condition is placed after the LOOP keyword, e.g.
    /// DO ... LOOP WHILE A > 0
    Bottom,
}

/// Specifies if a DO LOOP is using an
/// UNTIL or WHILE in its condition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DoLoopConditionKind {
    Until,
    While,
}
