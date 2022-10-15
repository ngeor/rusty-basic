use crate::linter::pre_linter::{HasFunctions, HasSubs, HasUserDefinedTypes};
use crate::linter::{FunctionMap, SubMap};
use crate::parser::UserDefinedTypes;

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

impl From<PreLinterResult> for UserDefinedTypes {
    fn from(pre_linter_result: PreLinterResult) -> Self {
        pre_linter_result.user_defined_types
    }
}
