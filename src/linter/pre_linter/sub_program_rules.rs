use crate::common::QErrorNode;
use crate::linter::pre_linter::can_pre_lint::CanPreLint;
use crate::linter::pre_linter::context::MainContextWithPos;
use crate::parser::{
    BareNameNode, FunctionImplementation, NameNode, ParamNameNodes, SubImplementation,
};

// function declaration
impl CanPreLint for (&NameNode, &ParamNameNodes) {
    type Context = MainContextWithPos;

    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        let (name_node, params) = *self;
        context
            .as_ref()
            .functions_mut()
            .add_declaration(name_node, params, context)
    }
}

impl CanPreLint for FunctionImplementation {
    type Context = MainContextWithPos;
    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        let Self { name, params, .. } = self;
        context
            .as_ref()
            .functions_mut()
            .add_implementation(name, params, context)
    }
}

// sub declaration
impl CanPreLint for (&BareNameNode, &ParamNameNodes) {
    type Context = MainContextWithPos;
    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        let (name_node, params) = *self;
        context
            .as_ref()
            .subs_mut()
            .add_declaration(name_node, params, context)
    }
}

impl CanPreLint for SubImplementation {
    type Context = MainContextWithPos;
    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        let Self { name, params, .. } = self;
        context
            .as_ref()
            .subs_mut()
            .add_implementation(name, params, context)
    }
}
