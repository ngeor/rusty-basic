use rusty_parser::UserDefinedTypes;

use crate::{
    core::{HasFunctions, HasSubs, SignatureMap},
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

    pub fn unwrap(self) -> (SignatureMap, SignatureMap, UserDefinedTypes) {
        (self.functions, self.subs, self.user_defined_types)
    }
}

impl HasFunctions for PreLinterResult {
    fn functions(&self) -> &SignatureMap {
        &self.functions
    }
}

impl HasSubs for PreLinterResult {
    fn subs(&self) -> &SignatureMap {
        &self.subs
    }
}

impl HasUserDefinedTypes for PreLinterResult {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        &self.user_defined_types
    }
}
