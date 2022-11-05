use crate::const_value_resolver::ConstValueResolver;
use crate::pre_linter::ConstantMap;
use crate::CastVariant;
use rusty_common::{Positioned, QError, QErrorPos, WithErrAt};
use rusty_parser::{BareName, ExpressionPos, Name, NamePos};

// calculate global constant values
pub fn global_const(
    global_constants: &mut ConstantMap,
    name_pos: &NamePos,
    expression_pos: &ExpressionPos,
) -> Result<(), QErrorPos> {
    let Positioned { element: name, pos } = name_pos;
    let bare_name: &BareName = name.bare_name();
    (match global_constants.get(bare_name) {
        Some(_) => Err(QError::DuplicateDefinition).with_err_at(pos),
        _ => Ok(()),
    })
    .and_then(|_| global_constants.resolve_const(expression_pos))
    .and_then(|v| match name {
        Name::Bare(_) => Ok(v),
        Name::Qualified(_, qualifier) => v.cast(*qualifier).with_err_at(expression_pos),
    })
    .map(|casted| {
        global_constants.insert(bare_name.clone(), casted);
    })
}
