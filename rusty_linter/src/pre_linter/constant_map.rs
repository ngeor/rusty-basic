use crate::core::ConstLookup;
use rusty_common::CaseInsensitiveString;
use rusty_parser::BareName;
use rusty_variant::Variant;
use std::collections::HashMap;

pub type ConstantMap = HashMap<BareName, Variant>;

impl ConstLookup for ConstantMap {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.get(name)
    }
}
