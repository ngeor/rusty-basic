use super::TypeQualifier;
use crate::common::CaseInsensitiveString;

pub trait TypeResolver {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier;
}

pub trait HasQualifier {
    fn qualifier(&self) -> TypeQualifier;
}

pub trait ResolveIntoRef<TInto> {
    fn resolve_into<T: TypeResolver>(&self, resolver: &T) -> TInto;
}

pub trait ResolveInto<TInto> {
    fn resolve_into<T: TypeResolver>(self, resolver: &T) -> TInto;
}
