use rusty_parser::*;

use crate::core::*;
use crate::names::Names;
use crate::pre_linter::PreLinterResult;

pub struct Context {
    pub functions: SignatureMap,
    pub subs: SignatureMap,
    pub user_defined_types: UserDefinedTypes,
    pub resolver: TypeResolverImpl,
    pub names: Names,
}

impl TypeResolver for Context {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
    }
}

impl Context {
    pub fn new(pre_linter_result: PreLinterResult) -> Self {
        Self {
            functions: pre_linter_result.functions,
            subs: pre_linter_result.subs,
            user_defined_types: pre_linter_result.user_defined_types,
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
