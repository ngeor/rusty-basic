use crate::common::{Locatable, QError, QErrorNode, ToLocatableError};
use crate::linter::const_value_resolver::ConstValueResolver;
use crate::linter::pre_linter::can_pre_lint::CanPreLint;
use crate::linter::pre_linter::context::MainContextWithPos;
use crate::linter::pre_linter::ConstantMap;
use crate::parser::{BareName, ExpressionNode, Name, NameNode, Statement};

impl CanPreLint for Statement {
    type Context = MainContextWithPos;
    fn pre_lint(&self, context: &Self::Context) -> Result<(), QErrorNode> {
        match self {
            Self::Const(name_node, expression_node) => global_const(
                &mut context.as_ref().global_constants_mut(),
                name_node,
                expression_node,
            ),
            _ => Ok(()),
        }
    }
}

// calculate global constant values
fn global_const(
    global_constants: &mut ConstantMap,
    name_node: &NameNode,
    expression_node: &ExpressionNode,
) -> Result<(), QErrorNode> {
    let Locatable { element: name, pos } = name_node;
    let bare_name: &BareName = name.bare_name();
    (match global_constants.get(bare_name) {
        Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
        _ => Ok(()),
    })
    .and_then(|_| global_constants.resolve_const(expression_node))
    .and_then(|v| match name {
        Name::Bare(_) => Ok(v),
        Name::Qualified(_, qualifier) => v.cast(*qualifier).with_err_at(expression_node),
    })
    .map(|casted| {
        global_constants.insert(bare_name.clone(), casted);
        ()
    })
}
