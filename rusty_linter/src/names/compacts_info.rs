use std::collections::HashMap;

use rusty_parser::specific::{TypeQualifier, VariableInfo};

#[derive(Default)]
pub struct CompactsInfo(HashMap<TypeQualifier, VariableInfo>);

impl CompactsInfo {
    pub fn get(&self, qualifier: &TypeQualifier) -> Option<&VariableInfo> {
        self.0.get(qualifier)
    }

    pub fn insert(&mut self, qualifier: TypeQualifier, variable_info: VariableInfo) {
        self.0.insert(qualifier, variable_info);
    }

    pub fn values(&self) -> impl Iterator<Item = &VariableInfo> {
        self.0.values()
    }
}
