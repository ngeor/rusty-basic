use super::{HasBareName, NameNode, QualifiedNameNode, TypeResolver};
use crate::common::{CaseInsensitiveString, Location};

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDeclarationNode {
    pub name: NameNode,
    pub parameters: Vec<NameNode>,
    pub pos: Location,
}

impl FunctionDeclarationNode {
    pub fn new(
        name: NameNode,
        parameters: Vec<NameNode>,
        pos: Location,
    ) -> FunctionDeclarationNode {
        FunctionDeclarationNode {
            name,
            parameters,
            pos,
        }
    }

    pub fn resolve(self, resolver: &dyn TypeResolver) -> QualifiedFunctionDeclarationNode {
        QualifiedFunctionDeclarationNode::new(
            self.name.resolve(resolver),
            self.parameters
                .into_iter()
                .map(|p| p.resolve(resolver))
                .collect(),
            self.pos,
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct QualifiedFunctionDeclarationNode {
    pub name: QualifiedNameNode,
    pub parameters: Vec<QualifiedNameNode>,
    pub pos: Location,
}

impl QualifiedFunctionDeclarationNode {
    pub fn new(
        name: QualifiedNameNode,
        parameters: Vec<QualifiedNameNode>,
        pos: Location,
    ) -> QualifiedFunctionDeclarationNode {
        QualifiedFunctionDeclarationNode {
            name,
            parameters,
            pos,
        }
    }
}

impl HasBareName for QualifiedFunctionDeclarationNode {
    fn bare_name(&self) -> &CaseInsensitiveString {
        self.name.bare_name()
    }

    fn bare_name_into(self) -> CaseInsensitiveString {
        self.name.bare_name_into()
    }
}
