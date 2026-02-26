use std::collections::HashMap;

use rusty_parser::{BuiltInStyle, TypeQualifier};
use rusty_variant::Variant;

use crate::core::VariableInfo;
use crate::names::traits::SingleNameTrait;

/// Stores information about compact variables of the same name.
/// The name of the variable isn't stored in this struct.
/// With compact variables, it's possible to have the same name
/// but with different types e.g. `A$` and `A%`.
#[derive(Default)]
pub struct Compacts {
    map: HashMap<TypeQualifier, VariableInfo>,
}

impl SingleNameTrait for Compacts {
    fn get_compact(&self, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        self.map.get(&qualifier)
    }

    fn get_extended(&self) -> Option<&VariableInfo> {
        None
    }

    fn get_const_value(&self) -> Option<&Variant> {
        None
    }

    fn collect_var_info(&self, only_shared: bool) -> Vec<(BuiltInStyle, &VariableInfo)> {
        self.map
            .values()
            .filter(|v| v.shared || !only_shared)
            .map(|v| (BuiltInStyle::Compact, v))
            .collect()
    }

    fn insert_compact(&mut self, variable_info: VariableInfo) {
        let q = variable_info
            .expression_type
            .opt_qualifier()
            .expect("Should be resolved");

        // if it already exists, it should be a REDIM
        debug_assert!(match self.map.get(&q) {
            Some(existing_v) => {
                existing_v.redim_info.is_some()
            }
            None => {
                true
            }
        });

        self.map.insert(q, variable_info);
    }
}
