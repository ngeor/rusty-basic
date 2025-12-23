use crate::names::names_inner::NamesInner;
use crate::names::traits::ManyNamesTrait;
use crate::NameContext;
use crate::{const_value_resolver::ConstLookup, names::ImplicitVars};
use rusty_common::CaseInsensitiveString;
use rusty_parser::specific::{
    BareName, BuiltInStyle, HasExpressionType, RedimInfo, TypeQualifier,
    VarTypeIsExtended, VariableInfo,
};
use rusty_variant::Variant;

/*

Naming rules

1. It is possible to have multiple compact variables

e.g. A = 3.14 (resolves as A! by the default rules), A$ = "hello", A% = 1

2. A constant can be referenced either bare or by its correct qualifier

2b. A constant cannot co-exist with other symbols of the same name

3. A bare constant gets its qualifier from the expression and not from the type resolver

4. A constant in a subprogram can override a global constant

5. An extended variable can be referenced either bare or by its correct qualifier
5b. An extended variable cannot co-exist with other symbols of the same name
*/

pub struct Names {
    data: NamesOneLevel,
    parent: Option<Box<Self>>,
    current_function_name: Option<BareName>,
}

/// Stores the data relevant to one level only (i.e. global symbols, or a FUNCTION, or a SUB).
/// Collects constant and variable names in [NamesInner] and implicit variables in [ImplicitVars].
/// TODO merge [ImplicitVars] into [NamesInner]
#[derive(Default)]
struct NamesOneLevel(NamesInner, ImplicitVars);

impl Names {
    pub fn new(parent: Option<Box<Self>>, current_function_name: Option<BareName>) -> Self {
        Self {
            data: NamesOneLevel::default(),
            parent,
            current_function_name,
        }
    }

    pub fn new_root() -> Self {
        Self::new(None, None)
    }

    pub fn get_implicit_vars_mut(&mut self) -> &mut ImplicitVars {
        &mut self.data.1
    }

    pub fn names(&self) -> &NamesInner {
        &self.data.0
    }

    pub fn names_mut(&mut self) -> &mut NamesInner {
        &mut self.data.0
    }

    pub fn get_compact_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match self.names().get_compact(bare_name, qualifier) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    fn get_compact_shared_var_recursively(
        &self,
        bare_name: &BareName,
        qualifier: TypeQualifier,
    ) -> Option<&VariableInfo> {
        match Self::require_shared(self.names().get_compact(bare_name, qualifier)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => {
                    parent_names.get_compact_shared_var_recursively(bare_name, qualifier)
                }
                _ => None,
            },
        }
    }

    pub fn get_extended_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match self.names().get_extended(bare_name) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    fn get_extended_shared_var_recursively(&self, bare_name: &BareName) -> Option<&VariableInfo> {
        match Self::require_shared(self.names().get_extended(bare_name)) {
            Some(variable_info) => Some(variable_info),
            _ => match &self.parent {
                Some(parent_names) => parent_names.get_extended_shared_var_recursively(bare_name),
                _ => None,
            },
        }
    }

    fn require_shared(opt_variable_info: Option<&VariableInfo>) -> Option<&VariableInfo> {
        match opt_variable_info {
            Some(variable_info) => {
                if variable_info.shared {
                    opt_variable_info
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    pub fn contains_any_locally_or_contains_extended_recursively(
        &self,
        bare_name: &BareName,
    ) -> bool {
        self.names().contains_key(bare_name)
            || self.get_extended_var_recursively(bare_name).is_some()
    }

    pub fn get_const_value_recursively(&self, bare_name: &BareName) -> Option<&Variant> {
        match self.names().get_const_value(bare_name) {
            Some(v) => Some(v),
            _ => {
                if let Some(boxed_parent) = &self.parent {
                    boxed_parent.get_const_value_recursively(bare_name)
                } else {
                    None
                }
            }
        }
    }

    pub fn contains_const_recursively(&self, bare_name: &BareName) -> bool {
        if self.names().get_const_value(bare_name).is_some() {
            true
        } else if let Some(boxed_parent) = &self.parent {
            boxed_parent.contains_const_recursively(bare_name)
        } else {
            false
        }
    }

    pub fn insert<T: HasExpressionType + VarTypeIsExtended>(
        &mut self,
        bare_name: BareName,
        dim_type: &T,
        shared: bool,
        redim_info: Option<RedimInfo>,
    ) {
        let variable_info = VariableInfo {
            expression_type: dim_type.expression_type(),
            shared,
            redim_info,
        };
        if dim_type.is_extended() {
            self.names_mut().insert_extended(bare_name, variable_info)
        } else {
            self.names_mut().insert_compact(bare_name, variable_info)
        }
    }

    pub fn is_in_function(&self, function_name: &BareName) -> bool {
        match &self.current_function_name {
            Some(f) => f == function_name,
            _ => false,
        }
    }

    pub fn is_in_subprogram(&self) -> bool {
        self.parent.is_some()
    }

    pub fn get_name_context(&self) -> NameContext {
        if self.parent.is_some() {
            if self.current_function_name.is_some() {
                NameContext::Function
            } else {
                NameContext::Sub
            }
        } else {
            NameContext::Global
        }
    }

    pub fn pop_parent(self) -> Option<Self> {
        self.parent.map(|boxed_parent| *boxed_parent)
    }

    pub fn find_name_or_shared_in_parent(
        &self,
        bare_name: &BareName,
    ) -> Vec<(BuiltInStyle, &VariableInfo)> {
        self.find_name(bare_name, false)
    }

    fn find_name(
        &self,
        bare_name: &BareName,
        only_shared: bool,
    ) -> Vec<(BuiltInStyle, &VariableInfo)> {
        let mut result = self.names().collect_var_info(bare_name, only_shared);

        if let Some(boxed_parent) = &self.parent {
            result.extend(boxed_parent.find_name(bare_name, true).into_iter());
        }
        result
    }
}

impl ConstLookup for Names {
    fn get_resolved_constant(&self, name: &CaseInsensitiveString) -> Option<&Variant> {
        match self.names().get_const_value(name) {
            Some(v) => Some(v),
            _ => match &self.parent {
                Some(boxed_parent) => boxed_parent.get_resolved_constant(name),
                _ => None,
            },
        }
    }
}
