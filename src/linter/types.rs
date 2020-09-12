use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    FileHandle, HasLocation, Locatable, Location, QError, QErrorNode, ToLocatableError,
};
use crate::parser::{
    BareName, BareNameNode, CanCastTo, HasQualifier, Operator, QualifiedName, QualifiedNameNode,
    TypeDefinition, TypeQualifier, UnaryOperator,
};

// TODO store the resolved type definition inside the expression at the time of the conversion from parser,
// in order to avoid `try_type_definition` all the time. A linter expression should have a resolved type definition.

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SingleLiteral(f32),
    DoubleLiteral(f64),
    StringLiteral(String),
    IntegerLiteral(i32),
    LongLiteral(i64),
    Constant(QualifiedName),
    Variable(ResolvedDeclaredNames),
    FunctionCall(QualifiedName, Vec<ExpressionNode>),
    BuiltInFunctionCall(BuiltInFunction, Vec<ExpressionNode>),
    BinaryExpression(Operator, Box<ExpressionNode>, Box<ExpressionNode>),
    UnaryExpression(UnaryOperator, Box<ExpressionNode>),
    Parenthesis(Box<ExpressionNode>),
    FileHandle(FileHandle),
}

impl Expression {
    pub fn try_type_definition(&self, pos: Location) -> Result<ResolvedTypeDefinition, QErrorNode> {
        match self {
            Self::SingleLiteral(_) => {
                Ok(ResolvedTypeDefinition::BuiltIn(TypeQualifier::BangSingle))
            }
            Self::DoubleLiteral(_) => {
                Ok(ResolvedTypeDefinition::BuiltIn(TypeQualifier::HashDouble))
            }
            Self::StringLiteral(_) => {
                Ok(ResolvedTypeDefinition::BuiltIn(TypeQualifier::DollarString))
            }
            Self::IntegerLiteral(_) => Ok(ResolvedTypeDefinition::BuiltIn(
                TypeQualifier::PercentInteger,
            )),
            Self::LongLiteral(_) => Ok(ResolvedTypeDefinition::BuiltIn(
                TypeQualifier::AmpersandLong,
            )),
            Self::Variable(names) => {
                let ResolvedDeclaredName {
                    type_definition, ..
                } = names.last().unwrap();
                Ok(type_definition.clone())
            }
            Self::Constant(name) | Self::FunctionCall(name, _) => {
                Ok(ResolvedTypeDefinition::BuiltIn(name.qualifier()))
            }
            Self::BuiltInFunctionCall(f, _) => Ok(ResolvedTypeDefinition::BuiltIn(f.qualifier())),
            Self::BinaryExpression(op, l, r) => match l.as_ref().try_type_definition()? {
                ResolvedTypeDefinition::BuiltIn(q_left) => {
                    match r.as_ref().try_type_definition()? {
                        ResolvedTypeDefinition::BuiltIn(q_right) => {
                            match super::casting::cast_binary_op(*op, q_left, q_right) {
                                Some(q_result) => Ok(ResolvedTypeDefinition::BuiltIn(q_result)),
                                None => Err(QError::TypeMismatch).with_err_at(r.pos()),
                            }
                        }
                        ResolvedTypeDefinition::UserDefined(_) => {
                            Err(QError::TypeMismatch).with_err_at(pos)
                        }
                    }
                }
                ResolvedTypeDefinition::UserDefined(_) => {
                    Err(QError::TypeMismatch).with_err_at(pos)
                }
            },
            Self::UnaryExpression(op, c) => match c.as_ref().try_type_definition()? {
                ResolvedTypeDefinition::BuiltIn(q) => match super::casting::cast_unary_op(*op, q) {
                    Some(q) => Ok(ResolvedTypeDefinition::BuiltIn(q)),
                    None => Err(QError::TypeMismatch).with_err_at(pos),
                },
                ResolvedTypeDefinition::UserDefined(_) => {
                    Err(QError::TypeMismatch).with_err_at(pos)
                }
            },
            Self::Parenthesis(c) => c.as_ref().try_type_definition(),
            Self::FileHandle(_) => Ok(ResolvedTypeDefinition::BuiltIn(TypeQualifier::FileHandle)),
        }
    }
}

pub type ExpressionNode = Locatable<Expression>;

impl ExpressionNode {
    pub fn try_type_definition(&self) -> Result<ResolvedTypeDefinition, QErrorNode> {
        self.as_ref().try_type_definition(self.pos())
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
    Is(Operator, ExpressionNode),
    Range(ExpressionNode, ExpressionNode),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Statement {
    Assignment(ResolvedDeclaredNames, ExpressionNode),
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
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResolvedTypeDefinition {
    BuiltIn(TypeQualifier),
    UserDefined(BareName),
}

impl CanCastTo<&ResolvedTypeDefinition> for ResolvedTypeDefinition {
    fn can_cast_to(&self, other: &Self) -> bool {
        match self {
            Self::BuiltIn(q_left) => match other {
                Self::BuiltIn(q_right) => q_left.can_cast_to(*q_right),
                _ => false,
            },
            Self::UserDefined(u_left) => match other {
                Self::UserDefined(u_right) => u_left == u_right,
                _ => false,
            },
        }
    }
}

impl CanCastTo<TypeQualifier> for ResolvedTypeDefinition {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        match self {
            Self::BuiltIn(q_left) => q_left.can_cast_to(other),
            Self::UserDefined(_) => false,
        }
    }
}

impl CanCastTo<&ResolvedDeclaredNames> for ResolvedTypeDefinition {
    fn can_cast_to(&self, other: &ResolvedDeclaredNames) -> bool {
        let ResolvedDeclaredName {
            type_definition, ..
        } = other.last().expect("Should have at least one element");
        self.can_cast_to(type_definition)
    }
}

impl From<TypeDefinition> for ResolvedTypeDefinition {
    fn from(type_definition: TypeDefinition) -> Self {
        match type_definition {
            TypeDefinition::Bare => panic!("Unresolved bare type"), // as this is internal error, it is ok to panic
            TypeDefinition::CompactBuiltIn(q) | TypeDefinition::ExtendedBuiltIn(q) => {
                Self::BuiltIn(q)
            }
            TypeDefinition::UserDefined(bare_name) => Self::UserDefined(bare_name),
        }
    }
}

// ========================================================
// ResolvedDeclaredName
// ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ResolvedDeclaredName {
    pub name: BareName,
    pub type_definition: ResolvedTypeDefinition,
}

impl ResolvedDeclaredName {
    pub fn new<S: AsRef<str>>(name: S, type_definition: ResolvedTypeDefinition) -> Self {
        Self {
            name: BareName::new(name.as_ref().to_owned()),
            type_definition,
        }
    }

    pub fn single<S: AsRef<str>>(
        name: S,
        type_definition: ResolvedTypeDefinition,
    ) -> ResolvedDeclaredNames {
        vec![ResolvedDeclaredName::new(name, type_definition)]
    }
}

impl AsRef<BareName> for ResolvedDeclaredName {
    fn as_ref(&self) -> &BareName {
        &self.name
    }
}

pub type ResolvedDeclaredNames = Vec<ResolvedDeclaredName>;
pub type ResolvedDeclaredNameNode = Locatable<ResolvedDeclaredName>;
pub type ResolvedDeclaredNameNodes = Vec<ResolvedDeclaredNameNode>;

// ========================================================
// ResolvedUserDefinedType
// ========================================================

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedUserDefinedType {
    /// The name of the type
    pub name: BareName,
    /// The elements
    pub elements: Vec<ResolvedElement>,
}

impl ResolvedUserDefinedType {
    pub fn find_element(&self, element_name: &BareName) -> Option<&ResolvedElement> {
        self.elements.iter().find(|e| &e.name == element_name)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedElement {
    pub name: BareName,
    pub element_type: ResolvedElementType,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ResolvedElementType {
    Integer,
    Long,
    Single,
    Double,
    String(u32),
    UserDefined(BareName),
}
