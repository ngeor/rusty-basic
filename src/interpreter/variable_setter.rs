use super::{Interpreter, Result, Stdlib, TypeResolver, Variant};
use crate::common::{CaseInsensitiveString, HasLocation, Location};
use crate::parser::{Name, NameNode, QualifiedName};

pub trait VariableSetter<T> {
    fn set_variable(
        &mut self,
        variable_name: T,
        variable_value: Variant,
    ) -> Result<Option<Variant>>;
}

impl<S: Stdlib> VariableSetter<NameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: NameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        let pos = variable_name.location();
        let name = variable_name.element_into();
        _set_variable_name_pos(self, name, pos, variable_value)
    }
}

impl<T: VariableSetter<NameNode>> VariableSetter<&NameNode> for T {
    fn set_variable(
        &mut self,
        variable_name: &NameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        self.set_variable(variable_name.clone(), variable_value)
    }
}

fn _set_variable_name_pos<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    name: Name,
    pos: Location,
    variable_value: Variant,
) -> Result<Option<Variant>> {
    match name {
        Name::Bare(bare_name) => {
            _set_variable_bare_name_pos(interpreter, bare_name, pos, variable_value)
        }
        Name::Typed(q) => _set_variable_typed_pos(interpreter, q, pos, variable_value),
    }
}

fn _set_variable_bare_name_pos<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    name: CaseInsensitiveString,
    pos: Location,
    variable_value: Variant,
) -> Result<Option<Variant>> {
    match interpreter.context_ref().resolve_result_name_bare(name) {
        Name::Typed(result_name) => {
            // assigning the return value to a function using an unqualified name
            interpreter
                .context_mut()
                .cast_insert(result_name, variable_value, pos)
        }
        Name::Bare(bare_name) => {
            let effective_type_qualifier = interpreter.resolve(&bare_name);
            let qualified_name = QualifiedName::new(bare_name, effective_type_qualifier);
            interpreter
                .context_mut()
                .cast_insert(qualified_name, variable_value, pos)
        }
    }
}

fn _set_variable_typed_pos<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    qualified_name: QualifiedName,
    pos: Location,
    variable_value: Variant,
) -> Result<Option<Variant>> {
    // make sure that if the name matches the function name then the type matches too
    interpreter
        .context_ref()
        .resolve_result_name_typed(&qualified_name, pos)?;
    interpreter
        .context_mut()
        .cast_insert(qualified_name, variable_value, pos)
}
