use super::error::*;
use crate::common::*;
use crate::parser::*;
use std::convert::TryFrom;

pub type QNameNode = Locatable<QualifiedName>;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInFunction {
    /// ENVIRON$
    Environ,
    /// LEN
    Len,
    /// STR$
    Str,
    /// VAL
    Val,
}

impl From<&CaseInsensitiveString> for Option<BuiltInFunction> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInFunction> {
        if s == "ENVIRON" {
            Some(BuiltInFunction::Environ)
        } else if s == "LEN" {
            Some(BuiltInFunction::Len)
        } else if s == "STR" {
            Some(BuiltInFunction::Str)
        } else if s == "VAL" {
            Some(BuiltInFunction::Val)
        } else {
            None
        }
    }
}

impl TryFrom<&Name> for Option<BuiltInFunction> {
    type Error = Error;
    fn try_from(name: &Name) -> Result<Option<BuiltInFunction>, Self::Error> {
        let opt_built_in: Option<BuiltInFunction> = name.bare_name().into();
        match opt_built_in {
            Some(b) => match b {
                BuiltInFunction::Environ => {
                    // ENVIRON$ must be qualified
                    match name {
                        Name::Bare(_) => err_no_pos(LinterError::SyntaxError),
                        Name::Qualified(q) => {
                            if q.qualifier() == TypeQualifier::DollarString {
                                Ok(Some(b))
                            } else {
                                err_no_pos(LinterError::TypeMismatch)
                            }
                        }
                    }
                }
                BuiltInFunction::Len => {
                    // LEN must be unqualified
                    match name {
                        Name::Bare(_) => Ok(Some(b)),
                        Name::Qualified(_) => err_no_pos(LinterError::SyntaxError),
                    }
                }
                BuiltInFunction::Str => {
                    // STR$ or otherwise it's undefined
                    match name {
                        // confirmed that even with DEFSTR A-Z it won't work as unqualified
                        Name::Bare(_) => Ok(None),
                        Name::Qualified(q) => {
                            if q.qualifier() == TypeQualifier::DollarString {
                                Ok(Some(b))
                            } else {
                                Ok(None)
                            }
                        }
                    }
                }
                BuiltInFunction::Val => {
                    // VAL must be unqualified
                    match name {
                        Name::Bare(_) => Ok(Some(b)),
                        Name::Qualified(_) => err_no_pos(LinterError::SyntaxError),
                    }
                }
            },
            None => Ok(None),
        }
    }
}

impl HasQualifier for BuiltInFunction {
    fn qualifier(&self) -> TypeQualifier {
        match self {
            Self::Environ => TypeQualifier::DollarString,
            Self::Len => TypeQualifier::PercentInteger,
            Self::Str => TypeQualifier::DollarString,
            Self::Val => TypeQualifier::BangSingle,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BuiltInSub {
    Environ,
    Input,
    Print,
    System,
}

impl From<&CaseInsensitiveString> for Option<BuiltInSub> {
    fn from(s: &CaseInsensitiveString) -> Option<BuiltInSub> {
        if s == "ENVIRON" {
            Some(BuiltInSub::Environ)
        } else if s == "INPUT" {
            Some(BuiltInSub::Input)
        } else if s == "PRINT" {
            Some(BuiltInSub::Print)
        } else if s == "SYSTEM" {
            Some(BuiltInSub::System)
        } else {
            None
        }
    }
}

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
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(Operand, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperand, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
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
            Self::BuiltInFunctionCall(f, _) => Ok(f.qualifier()),
            Self::BinaryExpression(op, l, r) => {
                let q_left = l.as_ref().as_ref().try_qualifier()?;
                let q_right = r.as_ref().as_ref().try_qualifier()?;
                if q_left.can_cast_to(q_right) {
                    match op {
                        Operand::Plus | Operand::Minus => Ok(q_left),
                        Operand::Less
                        | Operand::LessOrEqual
                        | Operand::Equal
                        | Operand::GreaterOrEqual
                        | Operand::Greater => Ok(TypeQualifier::PercentInteger),
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
            Self::Parenthesis(c) => c.as_ref().as_ref().try_qualifier(),
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
