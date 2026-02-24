use rusty_parser::UserDefinedTypes;

use crate::core::SignatureMap;

/// Stores the result of the pre-linter.
pub struct PreLinterResult {
    pub functions: SignatureMap,
    pub subs: SignatureMap,
    pub user_defined_types: UserDefinedTypes,
}

impl From<PreLinterResult> for UserDefinedTypes {
    fn from(value: PreLinterResult) -> Self {
        value.user_defined_types
    }
}
