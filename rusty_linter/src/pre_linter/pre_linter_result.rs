use crate::core::{FunctionMap, HasFunctions, HasSubs, HasUserDefinedTypes, SubMap};
use rusty_parser::UserDefinedTypes;

/// Stores the result of the pre-linter.
pub struct PreLinterResult {
    functions: FunctionMap,
    subs: SubMap,
    user_defined_types: UserDefinedTypes,
}

impl PreLinterResult {
    pub fn new(functions: FunctionMap, subs: SubMap, user_defined_types: UserDefinedTypes) -> Self {
        Self {
            functions,
            subs,
            user_defined_types,
        }
    }

    pub fn unwrap(self) -> (FunctionMap, SubMap, UserDefinedTypes) {
        (self.functions, self.subs, self.user_defined_types)
    }
}

impl HasFunctions for PreLinterResult {
    fn functions(&self) -> &FunctionMap {
        &self.functions
    }
}

impl HasSubs for PreLinterResult {
    fn subs(&self) -> &SubMap {
        &self.subs
    }
}

impl HasUserDefinedTypes for PreLinterResult {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        &self.user_defined_types
    }
}
