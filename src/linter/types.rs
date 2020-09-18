mod expression;
mod has_type_definition;
mod type_definition;
mod user_defined_type;

pub use self::expression::*;
pub use self::has_type_definition::*;
pub use self::type_definition::*;
pub use self::user_defined_type::*;

use crate::built_ins::BuiltInSub;
use crate::common::{CanCastTo, Locatable};
use crate::parser::{
    BareName, BareNameNode, Operator, QualifiedName, QualifiedNameNode, TypeQualifier,
};

#[cfg(test)]
use std::convert::TryFrom;

#[derive(Clone, Debug, PartialEq)]
pub struct ForLoopNode {
    pub variable_name: ResolvedDeclaredName,
    pub lower_bound: ExpressionNode,
    pub upper_bound: ExpressionNode,
    pub step: Option<ExpressionNode>,
    pub statements: StatementNodes,
    pub next_counter: Option<Locatable<ResolvedDeclaredName>>,
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

impl CanCastTo<TypeQualifier> for ResolvedDeclaredName {
    fn can_cast_to(&self, other: TypeQualifier) -> bool {
        self.type_definition().can_cast_to(other)
    }
}

// ========================================================
// ResolvedDeclaredName
// ========================================================

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct UserDefinedName {
    pub name: BareName,
    pub type_name: BareName,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Members {
    Leaf {
        name: BareName,
        element_type: ElementType,
    },
    Node(UserDefinedName, Box<Members>),
}

impl Members {
    pub fn name_path(&self) -> Vec<BareName> {
        match self {
            Self::Leaf { name, .. } => vec![name.clone()],
            Self::Node(UserDefinedName { name, .. }, boxed_members) => {
                let mut result = vec![name.clone()];
                result.extend(boxed_members.name_path());
                result
            }
        }
    }

    pub fn append(self, other: Self) -> Self {
        match self {
            Self::Leaf { name, element_type } => match element_type {
                ElementType::UserDefined(type_name) => {
                    Self::Node(UserDefinedName { name, type_name }, Box::new(other))
                }
                _ => panic!("Cannot extend leaf element which is not user defined type"),
            },
            Self::Node(user_defined_name, boxed_members) => {
                Self::Node(user_defined_name, Box::new(boxed_members.append(other)))
            }
        }
    }
}

impl HasTypeDefinition for Members {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::Leaf { element_type, .. } => element_type.type_definition(),
            Self::Node(_, boxed_members) => boxed_members.type_definition(),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum ResolvedDeclaredName {
    // A -> A!
    // A AS STRING
    // A$, A% etc
    BuiltIn(QualifiedName),

    // DIM C AS Card
    UserDefined(UserDefinedName),

    // C.Suit, Name.Address, Name.Address.PostCode
    Many(UserDefinedName, Members),
}

impl ResolvedDeclaredName {
    #[cfg(test)]
    pub fn parse(s: &str) -> Self {
        Self::BuiltIn(QualifiedName::try_from(s).unwrap())
    }

    #[cfg(test)]
    pub fn user_defined(name: &str, type_name: &str) -> Self {
        Self::UserDefined(UserDefinedName {
            name: name.into(),
            type_name: type_name.into(),
        })
    }

    pub fn name_path(&self) -> Vec<BareName> {
        match self {
            Self::BuiltIn(QualifiedName { name, .. }) => vec![name.clone()],
            Self::UserDefined(UserDefinedName { name, .. }) => vec![name.clone()],
            Self::Many(UserDefinedName { name, .. }, members) => {
                let mut result = vec![name.clone()];
                result.extend(members.name_path());
                result
            }
        }
    }

    pub fn append(self, members: Members) -> Self {
        match self {
            Self::BuiltIn(_) => panic!("Cannot append members to built-in resolved name"),
            Self::UserDefined(user_defined_name) => Self::Many(user_defined_name, members),
            Self::Many(user_defined_name, existing_members) => {
                Self::Many(user_defined_name, existing_members.append(members))
            }
        }
    }
}

impl AsRef<BareName> for ResolvedDeclaredName {
    fn as_ref(&self) -> &BareName {
        match self {
            Self::BuiltIn(QualifiedName { name, .. }) => name,
            Self::UserDefined(UserDefinedName { name, .. }) => name,
            Self::Many(UserDefinedName { name, .. }, _) => name,
        }
    }
}

impl HasTypeDefinition for ResolvedDeclaredName {
    fn type_definition(&self) -> TypeDefinition {
        match self {
            Self::BuiltIn(QualifiedName { qualifier, .. }) => TypeDefinition::BuiltIn(*qualifier),
            Self::UserDefined(UserDefinedName { type_name, .. }) => {
                TypeDefinition::UserDefined(type_name.clone())
            }
            Self::Many(_, members) => members.type_definition(),
        }
    }
}

pub type ResolvedDeclaredNameNode = Locatable<ResolvedDeclaredName>;
pub type ResolvedDeclaredNameNodes = Vec<ResolvedDeclaredNameNode>;
