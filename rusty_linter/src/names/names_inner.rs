use std::collections::HashMap;

use rusty_parser::{BareName, Name, TypeQualifier, VariableInfo};
use rusty_variant::Variant;

use crate::names::{
    name_info::NameInfo,
    traits::{ManyNamesTrait, SingleNameTrait},
};

/// Stores information about multiple constants or variable names.
/// This struct does not support multiple levels (e.g. `FUNCTION` or `SUB`).
#[derive(Default)]
pub struct NamesInner(HashMap<BareName, NameInfo>);

impl NamesInner {
    /// Returns true if this name is a constant, or an extended variable,
    /// or a compact variable. In the case of compact variables, multiple may
    /// exist with the same bare name, e.g. `A$` and `A%`.
    pub fn contains_key(&self, bare_name: &BareName) -> bool {
        self.0.contains_key(bare_name)
    }

    pub fn insert_const(&mut self, bare_name: BareName, v: Variant) {
        debug_assert!(!self.0.contains_key(&bare_name));
        self.0.insert(bare_name, NameInfo::constant(v));
    }

    pub fn insert_extended(&mut self, bare_name: BareName, variable_context: VariableInfo) {
        // if it exists as REDIM extended, it's okay
        // all other cases where it already exists are not okay
        debug_assert!(match self.0.get(&bare_name) {
            Some(name_info) => {
                match name_info.get_extended() {
                    Some(e) => e.redim_info.is_some(),
                    _ => false,
                }
            }
            None => {
                true
            }
        });
        self.0
            .insert(bare_name, NameInfo::extended(variable_context));
    }

    pub fn get_variable_info_by_name(&self, name: &Name) -> Option<&VariableInfo> {
        match name {
            // if it's bare, then it has to be extended
            Name::Bare(bare_name) => self.get_extended(bare_name),
            // if it's qualified, it can be either one (e.g. A$ or A AS STRING)
            Name::Qualified(bare_name, qualifier) => self
                .get_compact(bare_name, *qualifier)
                .or_else(|| self.get_extended(bare_name)),
        }
    }
}

impl ManyNamesTrait for NamesInner {
    fn get_compact(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&VariableInfo> {
        self.0
            .get(bare_name)
            .and_then(|name_info| name_info.get_compact(qualifier))
    }

    fn get_extended(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        self.0
            .get(bare_name)
            .and_then(|name_info| name_info.get_extended())
    }

    fn get_const_value(&self, bare_name: &BareName) -> Option<&rusty_variant::Variant> {
        self.0
            .get(bare_name)
            .and_then(|name_info| name_info.get_const_value())
    }

    fn collect_var_info(
        &self,
        bare_name: &BareName,
        only_shared: bool,
    ) -> Vec<(rusty_parser::BuiltInStyle, &VariableInfo)> {
        self.0
            .get(bare_name)
            .map(|name_info| name_info.collect_var_info(only_shared))
            .unwrap_or_default()
    }

    fn insert_compact(&mut self, bare_name: BareName, variable_info: VariableInfo) {
        match self.0.get_mut(&bare_name) {
            Some(name_info) => {
                name_info.insert_compact(variable_info);
            }
            None => {
                let mut name_info = NameInfo::compacts();
                name_info.insert_compact(variable_info);
                self.0.insert(bare_name, name_info);
            }
        }
    }
}
