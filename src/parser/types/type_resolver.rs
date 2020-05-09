use super::{NameTrait, TypeQualifier};

pub trait TypeResolver {
    fn resolve<T: NameTrait>(&self, name: &T) -> TypeQualifier;
}
