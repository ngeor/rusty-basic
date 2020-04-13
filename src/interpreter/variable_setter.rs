use super::{Interpreter, Result, Stdlib, TypeResolver, Variant};
use crate::common::{CaseInsensitiveString, HasLocation, Location};
use crate::parser::{Name, NameNode, QualifiedName};

//
// VariableSetter
//

pub trait VariableSetter<T> {
    fn set_variable(
        &mut self,
        variable_name: T,
        variable_value: Variant,
    ) -> Result<Option<Variant>>;
}

impl<S: Stdlib> VariableSetter<&NameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: &NameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        self.set_variable(variable_name.clone(), variable_value)
    }
}

impl<S: Stdlib> VariableSetter<NameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: NameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        let pos = variable_name.location();
        let x = (variable_name.element_into(), pos);
        self.set_variable(x, variable_value)
    }
}

impl<S: Stdlib> VariableSetter<(Name, Location)> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: (Name, Location),
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        let (name, pos) = variable_name;
        match name {
            Name::Bare(b) => self.set_variable((b, pos), variable_value),
            Name::Typed(t) => self.set_variable((t, pos), variable_value),
        }
    }
}

impl<S: Stdlib> VariableSetter<(QualifiedName, Location)> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: (QualifiedName, Location),
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        let (name, pos) = variable_name;
        self.context_ref().resolve_result_name_typed(&name, pos)?;
        self.context_mut().cast_insert(name, variable_value, pos)
    }
}

impl<S: Stdlib> VariableSetter<(CaseInsensitiveString, Location)> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: (CaseInsensitiveString, Location),
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        let (name, pos) = variable_name;
        match self.context_ref().resolve_result_name_bare(name) {
            Name::Typed(result_name) => {
                // assigning the return value to a function using an unqualified name
                self.context_mut()
                    .cast_insert(result_name, variable_value, pos)
            }
            Name::Bare(bare_name) => {
                let effective_type_qualifier = self.resolve(&bare_name);
                let qualified_name = QualifiedName::new(bare_name, effective_type_qualifier);
                self.context_mut()
                    .cast_insert(qualified_name, variable_value, pos)
            }
        }
    }
}
