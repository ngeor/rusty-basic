use crate::const_value_resolver::ConstValueResolver;
use crate::pre_linter::ConstantMap;
use rusty_common::{Locatable, QError, QErrorNode, ToLocatableError};
use rusty_parser::{BareName, ExpressionNode, Name, NameNode};

// calculate global constant values
pub fn global_const(
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
    })
}
