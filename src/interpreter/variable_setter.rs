use super::casting::cast;
use super::{Interpreter, InterpreterError, Result, Stdlib, TypeResolver, Variant};
use crate::common::{CaseInsensitiveString, Location};
use crate::parser::{HasQualifier, Name, NameNode, QualifiedName};

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
        let (name, pos) = variable_name.consume();
        match name {
            Name::Bare(bare_name) => for_bare::set(self, bare_name, pos, variable_value),
            Name::Typed(q) => for_typed::set(self, q, pos, variable_value),
        }
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

mod for_bare {
    use super::{cast_insert, resolve_and_set, Interpreter, Result, Stdlib, Variant};
    use crate::common::{CaseInsensitiveString, Location};
    use crate::interpreter::context::Context;

    pub fn set<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        name: CaseInsensitiveString,
        pos: Location,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        match interpreter.context_ref() {
            Context::Function(_, result_name, _) => {
                if result_name.bare_name() != &name {
                    // different names, it does not match with the result name
                    resolve_and_set(interpreter, name, pos, variable_value)
                } else {
                    // names match
                    // promote the bare name node to a qualified
                    let result_name_copy = result_name.clone();
                    cast_insert(interpreter, result_name_copy, variable_value, pos)
                }
            }
            _ => resolve_and_set(interpreter, name, pos, variable_value),
        }
    }
}

fn resolve_and_set<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    name: CaseInsensitiveString,
    pos: Location,
    value: Variant,
) -> Result<Option<Variant>> {
    let effective_type_qualifier = interpreter.resolve(&name);
    let qualified_name = QualifiedName::new(name, effective_type_qualifier);
    cast_insert(interpreter, qualified_name, value, pos)
}

fn cast_insert<S: Stdlib>(
    interpreter: &mut Interpreter<S>,
    name: QualifiedName,
    value: Variant,
    pos: Location,
) -> Result<Option<Variant>> {
    cast(value, name.qualifier())
        .map_err(|e| InterpreterError::new_with_pos(e, pos))
        .map(|casted| interpreter.context_mut().insert(name, casted))
}

mod for_typed {
    use super::{cast_insert, Interpreter, InterpreterError, Result, Stdlib, Variant};
    use crate::common::Location;
    use crate::interpreter::context::Context;
    use crate::parser::{HasQualifier, QualifiedName};

    pub fn set<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        qualified_name: QualifiedName,
        pos: Location,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        // make sure that if the name matches the function name then the type matches too
        match interpreter.context_ref() {
            Context::Function(_, result_name, _) => {
                if result_name.bare_name() != qualified_name.bare_name() {
                    // different names, it does not match with the result name
                    cast_insert(interpreter, qualified_name, variable_value, pos)
                } else {
                    // names match
                    if qualified_name.qualifier() == result_name.qualifier() {
                        cast_insert(interpreter, qualified_name, variable_value, pos)
                    } else {
                        Err(InterpreterError::new_with_pos("Duplicate definition", pos))
                    }
                }
            }
            _ => cast_insert(interpreter, qualified_name, variable_value, pos),
        }
    }
}
