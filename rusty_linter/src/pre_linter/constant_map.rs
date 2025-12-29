use std::collections::HashMap;

use rusty_common::{AtPos, CaseInsensitiveString, HasPos};
use rusty_parser::{AsBareName, BareName, Constant, Name};
use rusty_variant::Variant;

use crate::core::*;

#[derive(Default)]
pub struct ConstantMap(HashMap<BareName, Variant>);

impl ConstLookup for ConstantMap {
    fn get_const_value(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.0.get(name)
    }
}

impl Visitor<Constant> for ConstantMap {
    // calculate global constant values
    fn visit(&mut self, element: &Constant) -> VisitResult {
        let (name_pos, expression_pos) = element.into();
        self.ensure_is_not_already_defined(name_pos)
            .and_then(|_| self.eval_const(expression_pos))
            .and_then(|v| {
                Self::cast_resolved_value_to_declared_type(v, &name_pos.element, expression_pos)
            })
            .map(|casted| {
                self.store_resolved_value(name_pos, casted);
            })
    }
}

impl ConstantMap {
    fn ensure_is_not_already_defined(&self, name: &(impl AsBareName + HasPos)) -> VisitResult {
        match self.0.get(name.as_bare_name()) {
            Some(_) => Err(LintError::DuplicateDefinition.at(name)),
            _ => Ok(()),
        }
    }

    fn cast_resolved_value_to_declared_type(
        resolved_value: Variant,
        name: &Name,
        expression_pos: &impl HasPos,
    ) -> Result<Variant, LintErrorPos> {
        match name.qualifier() {
            Some(qualifier) => resolved_value
                .cast(qualifier)
                .map_err(|e| e.at(expression_pos)),
            _ => Ok(resolved_value),
        }
    }

    fn store_resolved_value(&mut self, name: &impl AsBareName, resolved_value: Variant) {
        self.0.insert(name.as_bare_name().clone(), resolved_value);
    }
}
