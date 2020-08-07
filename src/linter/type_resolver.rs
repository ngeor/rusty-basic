use super::{DeclaredName, NameTrait, QualifiedName, TypeQualifier};
use std::convert::TryInto;

pub trait TypeResolver {
    fn resolve<T: NameTrait>(&self, name: &T) -> TypeQualifier;

    fn to_qualified_name<T: NameTrait>(&self, name: &T) -> QualifiedName {
        let q = self.resolve(name);
        name.with_type_ref(q)
    }
}

pub trait ResolveFrom<TFrom> {
    fn resolve_from<TResolver: TypeResolver>(x: TFrom, resolver: &TResolver) -> Self;
}

pub trait ResolveInto<TInto> {
    fn resolve_into<TResolver: TypeResolver>(self, resolver: &TResolver) -> TInto;
}

impl<TFrom, TInto> ResolveInto<TInto> for TFrom
where
    TInto: ResolveFrom<TFrom>,
{
    fn resolve_into<TResolver: TypeResolver>(self, resolver: &TResolver) -> TInto {
        TInto::resolve_from(self, resolver)
    }
}

impl<T: NameTrait> ResolveFrom<&T> for TypeQualifier {
    fn resolve_from<TResolver: TypeResolver>(n: &T, resolver: &TResolver) -> Self {
        resolver.resolve(n)
    }
}

impl<T: NameTrait> ResolveFrom<T> for QualifiedName {
    fn resolve_from<TResolver: TypeResolver>(n: T, resolver: &TResolver) -> Self {
        let q = resolver.resolve(&n);
        n.with_type(q)
    }
}

impl ResolveFrom<&DeclaredName> for TypeQualifier {
    fn resolve_from<TR: TypeResolver>(declared_name: &DeclaredName, resolver: &TR) -> Self {
        if declared_name.is_bare() {
            resolver.resolve(declared_name.bare_name())
        } else {
            declared_name
                .try_into()
                .expect("Not implemented for user defined types")
        }
    }
}

impl ResolveFrom<&DeclaredName> for QualifiedName {
    fn resolve_from<TR: TypeResolver>(declared_name: &DeclaredName, resolver: &TR) -> Self {
        let q: TypeQualifier = declared_name.resolve_into(resolver);
        Self::new(declared_name.bare_name().clone(), q)
    }
}
