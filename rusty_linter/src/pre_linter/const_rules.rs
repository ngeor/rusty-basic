use crate::core::CastVariant;
use crate::core::ConstValueResolver;
use crate::core::{LintError, LintErrorPos};
use crate::pre_linter::ConstantMap;
use rusty_common::{AtPos, Positioned};
use rusty_parser::specific::{BareName, ExpressionPos, Name, NamePos};

// calculate global constant values
pub fn global_const(
    global_constants: &mut ConstantMap,
    name_pos: &NamePos,
    expression_pos: &ExpressionPos,
) -> Result<(), LintErrorPos> {
    let Positioned { element: name, pos } = name_pos;
    let bare_name: &BareName = name.bare_name();
    (match global_constants.get(bare_name) {
        Some(_) => Err(LintError::DuplicateDefinition.at(pos)),
        _ => Ok(()),
    })
    .and_then(|_| global_constants.resolve_const(expression_pos))
    .and_then(|v| match name {
        Name::Bare(_) => Ok(v),
        Name::Qualified(_, qualifier) => v.cast(*qualifier).map_err(|e| e.at(expression_pos)),
    })
    .map(|casted| {
        global_constants.insert(bare_name.clone(), casted);
    })
}
