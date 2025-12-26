use crate::core::*;
use crate::names::Names;
use crate::pre_linter::PreLinterResult;
use rusty_parser::*;

pub struct Context {
    pre_linter_result: PreLinterResult,
    pub resolver: TypeResolverImpl,
    // TODO make this private
    pub names: Names,
}

impl TypeResolver for Context {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
    }
}

impl HasSubprograms for Context {
    fn functions(&self) -> &SignatureMap {
        self.pre_linter_result.functions()
    }
    fn subs(&self) -> &SignatureMap {
        self.pre_linter_result.subs()
    }
}

impl HasUserDefinedTypes for Context {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.pre_linter_result.user_defined_types()
    }
}

impl Context {
    pub fn new(pre_linter_result: PreLinterResult) -> Self {
        Self {
            pre_linter_result,
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
        self.functions()
            .get(bare_name)
            .and_then(|function_signature_pos| function_signature_pos.element.qualifier())
    }
}

impl From<Context> for (PreLinterResult, Names) {
    fn from(value: Context) -> Self {
        (value.pre_linter_result, value.names)
    }
}
