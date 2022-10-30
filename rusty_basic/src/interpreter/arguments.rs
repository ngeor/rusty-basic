use crate::instruction_generator::Path;
use rusty_parser::variant::Variant;
use rusty_parser::ParamName;
use std::slice::Iter;
use std::vec::IntoIter;

#[derive(Debug, Default)]
pub struct Arguments {
    v: Vec<ArgumentInfo>,
}

#[derive(Debug)]
pub struct ArgumentInfo {
    pub value: Variant,
    pub param_name: Option<ParamName>,
    pub arg_path: Option<Path>,
}

impl Arguments {
    pub fn push_unnamed_by_val(&mut self, arg: Variant) {
        self.v.push(ArgumentInfo {
            value: arg,
            param_name: None,
            arg_path: None,
        });
    }

    pub fn push_unnamed_by_ref(&mut self, arg: Variant, path: Path) {
        self.v.push(ArgumentInfo {
            value: arg,
            param_name: None,
            arg_path: Some(path),
        });
    }

    pub fn push_named(&mut self, parameter_name: ParamName, arg: Variant) {
        self.v.push(ArgumentInfo {
            value: arg,
            param_name: Some(parameter_name),
            arg_path: None,
        });
    }

    pub fn iter(&self) -> Iter<ArgumentInfo> {
        self.v.iter()
    }

    pub fn into_iter(self) -> IntoIter<ArgumentInfo> {
        self.v.into_iter()
    }
}
