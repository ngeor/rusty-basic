use crate::linter::{ParamName, ParamType};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Variables {
    name_to_index: HashMap<Name, usize>,
    values: Vec<Variant>,
}

impl Variables {
    pub fn new() -> Self {
        Self {
            name_to_index: HashMap::new(),
            values: Vec::new(),
        }
    }

    pub fn insert_built_in(
        &mut self,
        bare_name: BareName,
        qualifier: TypeQualifier,
        value: Variant,
    ) {
        self.insert(QualifiedName::new(bare_name, qualifier).into(), value);
    }

    pub fn insert_user_defined(&mut self, bare_name: BareName, value: Variant) {
        self.insert(bare_name.into(), value);
    }

    pub fn insert_unnamed(&mut self, value: Variant) {
        self.values.push(value);
    }

    pub fn insert_param(&mut self, param_name: ParamName, value: Variant) {
        let (bare_name, param_type) = param_name.into_inner();
        match param_type {
            ParamType::BuiltIn(q) => self.insert_built_in(bare_name, q, value),
            ParamType::UserDefined(_) => self.insert_user_defined(bare_name, value),
        }
    }

    pub fn insert(&mut self, name: Name, value: Variant) {
        match self.name_to_index.get(&name) {
            Some(idx) => {
                self.values[*idx] = value;
            }
            None => {
                self.name_to_index.insert(name, self.values.len());
                self.values.push(value);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn get_built_in(&self, bare_name: &BareName, qualifier: TypeQualifier) -> Option<&Variant> {
        self.get_by_name(&QualifiedName::new(bare_name.clone(), qualifier).into())
    }

    pub fn get_user_defined(&self, bare_name: &BareName) -> Option<&Variant> {
        self.get_by_name(&bare_name.clone().into())
    }

    pub fn get_user_defined_mut(&mut self, bare_name: &BareName) -> Option<&mut Variant> {
        self.get_by_name_mut(&bare_name.clone().into())
    }

    fn get_by_name(&self, name: &Name) -> Option<&Variant> {
        self.name_to_index.get(name).and_then(|idx| self.get(*idx))
    }

    pub fn get_by_name_mut(&mut self, name: &Name) -> Option<&mut Variant> {
        match self.name_to_index.get(name) {
            Some(idx) => self.values.get_mut(*idx),
            None => None,
        }
    }

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.values.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.values.get_mut(idx)
    }
}
