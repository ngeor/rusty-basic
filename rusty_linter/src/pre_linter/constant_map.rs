use std::collections::HashMap;

use rusty_common::{AtPos, CaseInsensitiveString, Positioned};
use rusty_parser::{BareName, Constant, Name};
use rusty_variant::Variant;

use crate::core::*;

pub type ConstantMap = HashMap<BareName, Variant>;

impl ConstLookup for ConstantMap {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.get(name)
    }
}

impl Visitor<Constant> for ConstantMap {
    // calculate global constant values
    fn visit(&mut self, element: &Constant) -> VisitResult {
        let (name_pos, expression_pos) = element.into();
        let Positioned { element: name, pos } = name_pos;
        let bare_name: &BareName = name.bare_name();
        (match self.get(bare_name) {
            Some(_) => Err(LintError::DuplicateDefinition.at(pos)),
            _ => Ok(()),
        })
        .and_then(|_| self.resolve_const(expression_pos))
        .and_then(|v| match name {
            Name::Bare(_) => Ok(v),
            Name::Qualified(_, qualifier) => v.cast(*qualifier).map_err(|e| e.at(expression_pos)),
        })
        .map(|casted| {
            self.insert(bare_name.clone(), casted);
        })
    }
}
