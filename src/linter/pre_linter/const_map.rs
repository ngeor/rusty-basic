use crate::common::CaseInsensitiveString;
use crate::linter::const_value_resolver::ConstLookup;
use crate::parser::BareName;
use crate::variant::Variant;
use std::collections::HashMap;

pub type ConstantMap = HashMap<BareName, Variant>;

impl ConstLookup for ConstantMap {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.get(name)
    }
}
