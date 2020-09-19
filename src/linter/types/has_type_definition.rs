use super::TypeDefinition;
use crate::common::Locatable;

pub trait HasTypeDefinition {
    fn type_definition(&self) -> TypeDefinition;
}

impl<T: HasTypeDefinition> HasTypeDefinition for Locatable<T> {
    fn type_definition(&self) -> TypeDefinition {
        self.as_ref().type_definition()
    }
}
