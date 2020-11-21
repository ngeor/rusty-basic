use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};

pub trait TypeResolver {
    fn resolve_char(&self, ch: char) -> TypeQualifier;

    fn resolve(&self, bare_name: &BareName) -> TypeQualifier {
        let s: &str = bare_name.as_ref();
        let ch = s.chars().next().unwrap();
        self.resolve_char(ch)
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
}
