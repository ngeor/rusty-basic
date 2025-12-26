use rusty_parser::UserDefinedTypes;

use crate::{
    core::{HasSubprograms, SignatureMap},
    HasUserDefinedTypes,
};

/// Stores the result of the pre-linter.
pub struct PreLinterResult {
    functions: SignatureMap,
    subs: SignatureMap,
    user_defined_types: UserDefinedTypes,
}

impl PreLinterResult {
    pub fn new(
        functions: SignatureMap,
        subs: SignatureMap,
        user_defined_types: UserDefinedTypes,
    ) -> Self {
        Self {
            functions,
            subs,
            user_defined_types,
        }
    }
}

impl From<PreLinterResult> for UserDefinedTypes {
    fn from(value: PreLinterResult) -> Self {
        value.user_defined_types
    }
}

impl HasSubprograms for PreLinterResult {
    fn functions(&self) -> &SignatureMap {
        &self.functions
    }
    fn subs(&self) -> &SignatureMap {
        &self.subs
    }
}

impl HasUserDefinedTypes for PreLinterResult {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        &self.user_defined_types
    }
}
