use super::{NameTrait, QualifiedName, TypeQualifier};

pub trait TypeResolver {
    fn resolve<T: NameTrait>(&self, name: &T) -> TypeQualifier;

    fn to_qualified_name<T: NameTrait>(&self, name: &T) -> QualifiedName {
        let q = self.resolve(name);
        name.with_type_ref(q)
    }
}
