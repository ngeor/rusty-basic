use crate::{built_ins::built_in_sub::BuiltInSub, specific::*};
use rusty_common::*;

pub type StatementPos = Positioned<Statement>;
pub type Statements = Vec<StatementPos>;

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Comment(String),

    /// Assignment to a variable.
    ///
    /// Examples:
    ///
    /// ```bas
    /// A = 42
    /// A.Hello = 42
    /// A$ = "hello"
    /// ```
    ///
    /// The validity of the assignment is determined at the linting phase.
    Assignment(Expression, ExpressionPos),

    Const(NamePos, ExpressionPos),

    /// Declares a variable.
    ///
    /// Examples:
    ///
    /// ```bas
    /// DIM A                    ' (Bare)
    /// DIM A$                   ' (Compact)
    /// DIM A AS INTEGER         ' (ExtendedBuiltIn)
    /// DIM A AS STRING          ' (without length)
    /// DIM A AS STRING * 4      ' (with fixed length)
    /// DIM A AS UserDefinedType
    /// DIM SHARED A
    /// DIM A(1 TO 2)
    /// DIM A, B, C
    /// ```
    ///
    /// Parsing syntax reference:
    ///
    /// ```txt
    /// <dim> ::= "DIM"<ws><dim-name>
    /// <dim-name> ::= <bare-dim-name> | <compact-dim-name> | <extended-dim-name> | <user-defined-dim-name>
    ///
    /// (* DIM A, DIM A.B.C, DIM A.., DIM A(1 TO 2) *)
    /// <bare-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions>
    ///
    /// (* DIM A$, DIM A.B.C!, DIM A..% *)
    /// <compact-dim-name> ::= <compact-dim-name-left><opt-array-dimensions>
    /// (* it is allowed to have a keyword qualified by a string qualifier *)
    /// <compact-dim-name-left> ::= <bare-name-with-dots-not-keyword> ( "!" | "#" | "%" | "&")
    ///     | <bare-name-with-dots> "$"
    ///
    /// <extended-dim-name> ::= <bare-name-with-dots-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <extended-dim-type>
    /// <extended-dim-type> ::= "INTEGER"
    ///     | "LONG"
    ///     | "SINGLE"
    ///     | "DOUBLE"
    ///     | <extended-dim-string>
    /// <extended-dim-string> ::= "STRING" <opt-ws> "*" <opt-ws> <expression> | "STRING"
    ///
    /// (* user defined type variable cannot have dots *)
    /// <user-defined-dim-name> ::= <bare-name-not-keyword><opt-array-dimensions> <ws> "AS" <ws> <user-defined-type>
    /// <user-defined-type> ::= <bare-name-not-keyword>
    ///
    /// <opt-array-dimensions> ::= "" | "(" <opt-ws> <array-dimensions> <opt-ws> ")"
    /// <array-dimensions> ::= <array-dimension> | <array-dimension> <opt-ws> "," <opt-ws> <array-dimensions>
    /// <array-dimension> ::= <expression> | <expression> <ws> "TO" <ws> <expression>
    /// ```
    Dim(DimList),

    Redim(DimList),

    SubCall(BareName, Expressions),
    BuiltInSubCall(BuiltInSub, Expressions),

    /*
     * Decision flow
     */
    IfBlock(IfBlock),
    SelectCase(SelectCase),

    /*
     * Loops
     */
    ForLoop(ForLoop),
    While(ConditionalBlock),
    DoLoop(DoLoop),

    /*
     * Unstructured flow control
     */
    Label(CaseInsensitiveString),
    GoTo(CaseInsensitiveString),

    OnError(OnErrorOption),
    Resume(ResumeOption),

    GoSub(CaseInsensitiveString),
    Return(Option<CaseInsensitiveString>),

    Exit(ExitObject),

    End,
    System,

    /*
     * Special statements
     */
    Print(Print),
}

/// A list of variables defined in a DIM statement.
#[derive(Clone, Debug, PartialEq)]
pub struct DimList {
    /// Specifies if the variables are shared. Can only be used on the global
    /// module. If shared, the variables are available in functions/subs.
    pub shared: bool,

    /// The variables defined in the DIM statement.
    pub variables: DimVars,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ExitObject {
    Function,
    Sub,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResumeOption {
    Bare,
    Next,
    Label(CaseInsensitiveString),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OnErrorOption {
    Next,
    Label(CaseInsensitiveString),
    Zero,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoop {
    pub variable_name: ExpressionPos,
    pub lower_bound: ExpressionPos,
    pub upper_bound: ExpressionPos,
    pub step: Option<ExpressionPos>,
    pub statements: Statements,
    pub next_counter: Option<ExpressionPos>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ConditionalBlock {
    pub condition: ExpressionPos,
    pub statements: Statements,
}

#[derive(Clone, Debug, PartialEq)]
pub struct IfBlock {
    pub if_block: ConditionalBlock,
    pub else_if_blocks: Vec<ConditionalBlock>,
    pub else_block: Option<Statements>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SelectCase {
    /// The expression been matched
    pub expr: ExpressionPos,
    /// The case statements
    pub case_blocks: Vec<CaseBlock>,
    /// An optional CASE ELSE block
    pub else_block: Option<Statements>,
    /// Holds an optional inline comment after SELECT CASE X e.g. SELECT CASE X ' make a choice
    pub inline_comments: Vec<Positioned<String>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CaseBlock {
    pub expression_list: Vec<CaseExpression>,
    pub statements: Statements,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaseExpression {
    Simple(ExpressionPos),
    Is(Operator, ExpressionPos),
    Range(ExpressionPos, ExpressionPos),
}

#[derive(Clone, Debug, PartialEq)]
pub struct DoLoop {
    pub condition: ExpressionPos,
    pub statements: Statements,
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
