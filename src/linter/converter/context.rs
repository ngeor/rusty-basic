use crate::common::*;
use crate::linter::converter::names::Names;
use crate::linter::converter::traits::Convertible;
use crate::linter::converter::types::Implicits;
use crate::linter::pre_linter::PreLinterResult;
use crate::linter::type_resolver::{IntoQualified, TypeResolver};
use crate::linter::type_resolver_impl::TypeResolverImpl;
use crate::linter::{FunctionMap, HasFunctions, HasSubs, HasUserDefinedTypes, SubMap};
use crate::parser::*;
use std::rc::Rc;

pub struct Context {
    pre_linter_result: Rc<PreLinterResult>,
    pub resolver: TypeResolverImpl,
    pub names: Names,
}

impl TypeResolver for Context {
    fn char_to_qualifier(&self, ch: char) -> TypeQualifier {
        self.resolver.char_to_qualifier(ch)
    }
}

impl HasFunctions for Context {
    fn functions(&self) -> &FunctionMap {
        self.pre_linter_result.functions()
    }
}

impl HasSubs for Context {
    fn subs(&self) -> &SubMap {
        self.pre_linter_result.subs()
    }
}

impl HasUserDefinedTypes for Context {
    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.pre_linter_result.user_defined_types()
    }
}

impl Context {
    pub fn new(pre_linter_result: Rc<PreLinterResult>) -> Self {
        Self {
            pre_linter_result,
            resolver: TypeResolverImpl::new(),
            names: Names::new_root(),
        }
    }

    pub fn push_sub_context(
        &mut self,
        params: ParamNameNodes,
    ) -> Result<ParamNameNodes, QErrorNode> {
        let temp_dummy = Names::new_root();
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)), None);
        params.convert(self)
    }

    pub fn push_function_context(
        &mut self,
        name: Name,
        params: ParamNameNodes,
    ) -> Result<(Name, ParamNameNodes), QErrorNode> {
        let temp_dummy = Names::new_root();
        let old_names = std::mem::replace(&mut self.names, temp_dummy);
        self.names = Names::new(Some(Box::new(old_names)), Some(name.bare_name().clone()));
        let converted_function_name = name.to_qualified(self);
        Ok((converted_function_name, params.convert(self)?))
    }

    pub fn pop_context(&mut self) -> Implicits {
        // temp object for mem swap
        let temp_dummy = Names::new_root();
        // take current "self.names" and store into "current"
        let mut current = std::mem::replace(&mut self.names, temp_dummy);
        // collect implicits
        let mut implicits = Implicits::new();
        implicits.append(current.get_implicits());
        // set parent as current
        self.names = current.pop_parent().expect("Stack underflow");
        implicits
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.names.is_in_subprogram()
    }

    /// Gets the function qualifier of the function identified by the given bare name.
    /// If no such function exists, returns `None`.
    pub fn function_qualifier(&self, bare_name: &BareName) -> Option<TypeQualifier> {
        self.functions()
            .get(bare_name)
            .map(|function_signature_node| function_signature_node.as_ref().qualifier())
    }
}
