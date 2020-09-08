use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    FileHandle, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::parser::{
    BareName, BareNameNode, HasQualifier, Operand, QualifiedName, QualifiedNameNode,
    TypeDefinition, TypeQualifier, UnaryOperand,
};

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(ResolvedDeclaredName),
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
            Self::Variable(name) => {
                let ResolvedDeclaredName {
                    type_definition, ..
                } = name;
                match type_definition {
                    ResolvedTypeDefinition::CompactBuiltIn(q)
                    | ResolvedTypeDefinition::ExtendedBuiltIn(q) => Ok(*q),
                    _ => Err(QError::TypeMismatch).with_err_at(pos),
                }
            }
            Self::Constant(name) | Self::FunctionCall(name, _) => Ok(name.qualifier()),
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
    pub variable_name: ResolvedDeclaredNameNode,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<ResolvedDeclaredNameNode>,
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
    Assignment(ResolvedDeclaredName, ExpressionNode),
    Const(QualifiedNameNode, ExpressionNode),
    SubCall(BareName, Vec<ExpressionNode>),
    BuiltInSubCall(BuiltInSub, Vec<ExpressionNode>),

    IfBlock(IfBlockNode),
    SelectCase(SelectCaseNode),

    ForLoop(ForLoopNode),
    While(ConditionalBlockNode),

    ErrorHandler(BareName),
    Label(BareName),
    GoTo(BareName),

    SetReturnValue(ExpressionNode),
    Comment(String),
    Dim(ResolvedDeclaredNameNode),
}

pub type StatementNode = Locatable<Statement>;
pub type StatementNodes = Vec<StatementNode>;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionImplementation {
    pub name: QualifiedNameNode,
    pub params: ResolvedDeclaredNameNodes,
    pub body: StatementNodes,
}

#[derive(Clone, Debug, PartialEq)]
pub struct SubImplementation {
    pub name: BareNameNode,
    pub params: ResolvedDeclaredNameNodes,
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

// ========================================================
// ResolvedTypeDefinition
// ========================================================

/// Similar to `TypeDefinition` but without `Bare`
#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedTypeDefinition {
    CompactBuiltIn(TypeQualifier),
    ExtendedBuiltIn(TypeQualifier),
    UserDefined(BareName),
}

impl ResolvedTypeDefinition {
    pub fn is_compact_built_in(&self) -> bool {
        match self {
            Self::CompactBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_compact_of_type(&self, q: TypeQualifier) -> bool {
        match self {
            Self::CompactBuiltIn(q_self) => *q_self == q,
            _ => false,
        }
    }

    pub fn is_extended_built_in(&self) -> bool {
        match self {
            Self::ExtendedBuiltIn(_) => true,
            _ => false,
        }
    }

    pub fn is_user_defined(&self) -> bool {
        match self {
            Self::UserDefined(_) => true,
            _ => false,
        }
    }

    pub fn is_built_in(&self) -> bool {
        self.is_compact_built_in() || self.is_extended_built_in()
    }

    pub fn is_extended(&self) -> bool {
        self.is_extended_built_in() || self.is_user_defined()
    }
}

impl From<ResolvedTypeDefinition> for TypeDefinition {
    fn from(resolved_type_definition: ResolvedTypeDefinition) -> Self {
        match resolved_type_definition {
            ResolvedTypeDefinition::CompactBuiltIn(q) => Self::CompactBuiltIn(q),
            ResolvedTypeDefinition::ExtendedBuiltIn(q) => Self::ExtendedBuiltIn(q),
            ResolvedTypeDefinition::UserDefined(bare_name) => Self::UserDefined(bare_name),
        }
    }
}

impl From<TypeDefinition> for ResolvedTypeDefinition {
    fn from(type_definition: TypeDefinition) -> Self {
        match type_definition {
            TypeDefinition::Bare => panic!("Unresolved bare type"), // as this is internal error, it is ok to panic
            TypeDefinition::CompactBuiltIn(q) => Self::CompactBuiltIn(q),
            TypeDefinition::ExtendedBuiltIn(q) => Self::ExtendedBuiltIn(q),
            TypeDefinition::UserDefined(bare_name) => Self::UserDefined(bare_name),
        }
    }
}

// ========================================================
// ResolvedDeclaredName
// ========================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedDeclaredName {
    pub name: BareName,
    pub type_definition: ResolvedTypeDefinition,
}

impl AsRef<BareName> for ResolvedDeclaredName {
    fn as_ref(&self) -> &BareName {
        &self.name
    }
}

pub type ResolvedDeclaredNameNode = Locatable<ResolvedDeclaredName>;
pub type ResolvedDeclaredNameNodes = Vec<ResolvedDeclaredNameNode>;
