use crate::core::ConstLookup;
use crate::core::ResolvedParamType;
use rusty_common::CaseInsensitiveString;
use rusty_parser::specific::{BareName, TypeQualifier};
use rusty_variant::Variant;
use std::collections::HashMap;

pub type ConstantMap = HashMap<BareName, Variant>;

impl ConstLookup for ConstantMap {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        self.get(name)
    }
}

pub type ResolvedParamTypes = Vec<ResolvedParamType>;

#[derive(Eq, PartialEq)]
pub struct FunctionSignature {
    q: TypeQualifier,
    param_types: ResolvedParamTypes,
}

impl FunctionSignature {
    pub fn new(q: TypeQualifier, param_types: ResolvedParamTypes) -> Self {
        Self { q, param_types }
    }

    pub fn qualifier(&self) -> TypeQualifier {
        self.q
    }

    pub fn param_types(&self) -> &ResolvedParamTypes {
        &self.param_types
    }
}

#[derive(Eq, PartialEq)]
pub struct SubSignature {
    param_types: ResolvedParamTypes,
}

impl SubSignature {
    pub fn new(param_types: ResolvedParamTypes) -> Self {
        Self { param_types }
    }

    pub fn param_types(&self) -> &ResolvedParamTypes {
        &self.param_types
    }
}
