use crate::parser::{BareName, BareNameNode, Name, NameNode, QualifiedName, TypeQualifier};

pub trait TypeResolver {
    fn resolve<T: AsRef<str>>(&self, name: T) -> TypeQualifier;
}

// TODO deprecate these traits

pub trait ResolveFrom<TFrom> {
    fn resolve_from<TResolver: TypeResolver>(x: TFrom, resolver: &TResolver) -> Self;
}

pub trait ResolveInto<TInto> {
    fn resolve_into<TResolver: TypeResolver>(self, resolver: &TResolver) -> TInto;
}

// blanket ResolveInto implementation
impl<TFrom, TInto> ResolveInto<TInto> for TFrom
where
    TInto: ResolveFrom<TFrom>,
{
    fn resolve_into<TResolver: TypeResolver>(self, resolver: &TResolver) -> TInto {
        TInto::resolve_from(self, resolver)
    }
}

// &BareName -> TypeQualifier

impl ResolveFrom<&BareName> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: &BareName, resolver: &T) -> Self {
        resolver.resolve(x)
    }
}

// BareName -> TypeQualifier

impl ResolveFrom<BareName> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: BareName, resolver: &T) -> Self {
        resolver.resolve(x)
    }
}

// &BareNameNode -> TypeQualifier

impl ResolveFrom<&BareNameNode> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: &BareNameNode, resolver: &T) -> Self {
        let bare_name: &BareName = x.as_ref();
        bare_name.resolve_into(resolver)
    }
}

// &Name -> TypeQualifier

impl ResolveFrom<&Name> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: &Name, resolver: &T) -> Self {
        match x {
            Name::Bare(b) => b.resolve_into(resolver),
            Name::Qualified { qualifier, .. } => *qualifier,
        }
    }
}

// Name -> TypeQualifier

impl ResolveFrom<Name> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: Name, resolver: &T) -> Self {
        match x {
            Name::Bare(b) => b.resolve_into(resolver),
            Name::Qualified { qualifier, .. } => qualifier,
        }
    }
}

// &NameNode -> TypeQualifier

impl ResolveFrom<&NameNode> for TypeQualifier {
    fn resolve_from<T: TypeResolver>(x: &NameNode, resolver: &T) -> Self {
        let name: &Name = x.as_ref();
        name.resolve_into(resolver)
    }
}

impl<T: AsRef<BareName> + ResolveInto<TypeQualifier>> ResolveFrom<T> for QualifiedName {
    fn resolve_from<TR: TypeResolver>(name: T, resolver: &TR) -> Self {
        let n: BareName = name.as_ref().clone();
        let q: TypeQualifier = name.resolve_into(resolver);
        Self::new(n, q)
    }
}
