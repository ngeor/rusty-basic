use crate::parser::ParamName;
use crate::variant::Variant;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Debug)]
pub struct Arguments {
    v: Vec<(Option<ParamName>, Variant)>,
}

impl Arguments {
    pub fn new() -> Self {
        Self { v: vec![] }
    }

    pub fn push_unnamed(&mut self, arg: Variant) {
        self.v.push((None, arg));
    }

    pub fn push_named(&mut self, parameter_name: ParamName, arg: Variant) {
        self.v.push((Some(parameter_name), arg));
    }

    pub fn iter(&self) -> Iter<(Option<ParamName>, Variant)> {
        self.v.iter()
    }

    pub fn into_iter(self) -> IntoIter<(Option<ParamName>, Variant)> {
        self.v.into_iter()
    }
}
