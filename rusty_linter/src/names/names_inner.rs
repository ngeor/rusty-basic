use std::collections::HashMap;

use rusty_parser::specific::BareName;

use crate::names::name_info::NameInfo;

#[derive(Default)]
pub struct NamesInner(HashMap<BareName, NameInfo>);

impl NamesInner {
    pub fn contains_key(&self, bare_name: &BareName) -> bool {
        self.0.contains_key(bare_name)
    }

    pub fn get(&self, bare_name: &BareName) -> Option<&NameInfo> {
        self.0.get(bare_name)
    }

    pub fn get_mut(&mut self, bare_name: &BareName) -> Option<&mut NameInfo> {
        self.0.get_mut(bare_name)
    }

    pub fn insert(&mut self, bare_name: BareName, name_info: NameInfo) {
        self.0.insert(bare_name, name_info);
    }
}
