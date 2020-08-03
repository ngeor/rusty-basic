use crate::common::CaseInsensitiveString;
use crate::parser::{HasQualifier, NameTrait, QualifiedName, TypeQualifier};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct VariableSet(HashMap<CaseInsensitiveString, HashSet<TypeQualifier>>);

impl VariableSet {
    pub fn insert(&mut self, name: QualifiedName) {
        let (bare_name, qualifier) = name.consume();
        match self.0.get_mut(&bare_name) {
            Some(inner_set) => {
                inner_set.insert(qualifier);
            }
            None => {
                let mut inner_set: HashSet<TypeQualifier> = HashSet::new();
                inner_set.insert(qualifier);
                self.0.insert(bare_name, inner_set);
            }
        }
    }

    pub fn contains_qualified(&self, name: &QualifiedName) -> bool {
        match self.0.get(name.bare_name()) {
            Some(inner_set) => inner_set.contains(&name.qualifier()),
            None => false,
        }
    }

    pub fn contains_bare<U: NameTrait>(&self, name: &U) -> bool {
        self.0.contains_key(name.bare_name())
    }
}
