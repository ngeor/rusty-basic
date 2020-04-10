use super::{Interpreter, Result, Stdlib, TypeResolver, Variant};
use crate::parser::{BareNameNode, NameNode, QualifiedNameNode};

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
        match variable_name {
            NameNode::Bare(b) => self.set_variable(b, variable_value),
            NameNode::Typed(t) => self.set_variable(t, variable_value),
        }
    }
}

impl<S: Stdlib> VariableSetter<&QualifiedNameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: &QualifiedNameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        self.context_ref()
            .resolve_result_name_typed(variable_name)?;
        self.context_mut()
            .cast_insert(variable_name, variable_value)
    }
}

impl<S: Stdlib> VariableSetter<QualifiedNameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: QualifiedNameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        self.set_variable(&variable_name, variable_value)
    }
}

impl<S: Stdlib> VariableSetter<&BareNameNode> for Interpreter<S> {
    fn set_variable(
        &mut self,
        variable_name: &BareNameNode,
        variable_value: Variant,
    ) -> Result<Option<Variant>> {
        match self.context_ref().resolve_result_name_bare(variable_name) {
            Some(result_name) => {
                // assigning the return value to a function using an unqualified name
                self.context_mut().cast_insert(&result_name, variable_value)
            }
            None => {
                let effective_type_qualifier = self.resolve(variable_name.name());
                let qualified_name_node =
                    variable_name.to_qualified_name_node(effective_type_qualifier);
                self.context_mut()
                    .cast_insert(&qualified_name_node, variable_value)
            }
        }
    }
}
