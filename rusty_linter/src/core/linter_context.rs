use rusty_parser::*;

use crate::core::*;
use crate::names::Names;

pub struct LinterContext {
    pub functions: SignatureMap,
    pub subs: SignatureMap,
    pub user_defined_types: UserDefinedTypes,
    pub resolver: TypeResolverImpl,
    pub names: Names,
}

impl TypeResolver for LinterContext {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
    }
}

impl LinterContext {
    pub fn new(
        functions: SignatureMap,
        subs: SignatureMap,
        user_defined_types: UserDefinedTypes,
    ) -> Self {
        Self {
            functions,
            subs,
            user_defined_types,
            resolver: TypeResolverImpl::new(),
            names: Names::new(),
        }
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.names.is_in_subprogram()
    }

    /// Gets the function qualifier of the function identified by the given bare name.
    /// If no such function exists, returns `None`.
    pub fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions
            .get(bare_name)
            .and_then(|function_signature_pos| function_signature_pos.element.qualifier())
    }
}
