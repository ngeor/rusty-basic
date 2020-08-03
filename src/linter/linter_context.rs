use crate::common::CaseInsensitiveString;
use crate::linter::error::*;
use crate::linter::variable_set::VariableSet;
use crate::parser::{Name, NameTrait, TypeQualifier};
use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct LinterContext {
    parent: Option<Box<LinterContext>>,
    constants: HashMap<CaseInsensitiveString, TypeQualifier>,
    variables: VariableSet,
    function_name: Option<CaseInsensitiveString>,
    sub_name: Option<CaseInsensitiveString>,
}

impl LinterContext {
    pub fn push_function_context(self, name: &CaseInsensitiveString) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.function_name = Some(name.clone());
        result
    }

    pub fn push_sub_context(self, name: &CaseInsensitiveString) -> Self {
        let mut result = LinterContext::default();
        result.parent = Some(Box::new(self));
        result.sub_name = Some(name.clone());
        result
    }

    pub fn pop_context(self) -> Self {
        *self.parent.expect("Stack underflow!")
    }

    pub fn get_constant_type(&self, n: &Name) -> Result<Option<TypeQualifier>, Error> {
        let bare_name: &CaseInsensitiveString = n.bare_name();
        match self.constants.get(bare_name) {
            Some(const_type) => {
                // it's okay to reference a const unqualified
                if n.bare_or_eq(*const_type) {
                    Ok(Some(*const_type))
                } else {
                    Err(LinterError::DuplicateDefinition.into())
                }
            }
            None => Ok(None),
        }
    }

    pub fn get_parent_constant_type(&self, n: &Name) -> Result<Option<TypeQualifier>, Error> {
        match &self.parent {
            Some(p) => {
                let x = p.get_constant_type(n)?;
                match x {
                    Some(q) => Ok(Some(q)),
                    None => p.get_parent_constant_type(n),
                }
            }
            None => Ok(None),
        }
    }

    pub fn variables(&mut self) -> &mut VariableSet {
        &mut self.variables
    }

    pub fn constants(&mut self) -> &mut HashMap<CaseInsensitiveString, TypeQualifier> {
        &mut self.constants
    }

    pub fn is_function_context(&self, name: &Name) -> bool {
        match &self.function_name {
            Some(x) => x == name.bare_name(),
            None => false,
        }
    }
}
