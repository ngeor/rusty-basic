use crate::parser::{BareName, Name, NameRef, QualifiedName, TypeQualifier};

pub trait TypeResolver {
    fn resolve_char(&self, ch: char) -> TypeQualifier;

    fn resolve(&self, bare_name: &BareName) -> TypeQualifier {
        let s: &str = bare_name.as_ref();
        let ch = s.chars().next().unwrap();
        self.resolve_char(ch)
    }

    fn resolve_name_to_qualifier(&self, name: &Name) -> TypeQualifier {
        match name {
            Name::Bare(bare_name) => self.resolve(bare_name),
            Name::Qualified(QualifiedName { qualifier, .. }) => *qualifier,
        }
    }

    fn resolve_name(&self, name: &Name) -> QualifiedName {
        match name {
            Name::Bare(bare_name) => QualifiedName::new(bare_name.clone(), self.resolve(bare_name)),
            Name::Qualified(qualified_name) => qualified_name.clone(),
        }
    }

    fn resolve_name_to_name(&self, name: Name) -> Name {
        match name {
            Name::Bare(bare_name) => {
                let qualifier = self.resolve(&bare_name);
                Name::new(bare_name, Some(qualifier))
            }
            _ => name,
        }
    }

    fn resolve_name_ref_to_qualifier<'a, T>(&self, name: T) -> TypeQualifier
    where
        NameRef<'a>: From<T>,
    {
        let x = NameRef::from(name);
        match x.opt_q {
            Some(q) => q,
            _ => self.resolve(x.bare_name),
        }
    }

    fn resolve_name_to_name_ref<'a>(&self, name: &'a Name) -> NameRef<'a> {
        match name {
            Name::Bare(bare_name) => {
                let qualifier = self.resolve(bare_name);
                NameRef {
                    bare_name,
                    opt_q: Some(qualifier),
                }
            }
            Name::Qualified(QualifiedName {
                bare_name,
                qualifier,
            }) => NameRef {
                bare_name,
                opt_q: Some(*qualifier),
            },
        }
    }
}
