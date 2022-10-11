use crate::common::Locatable;
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier, VarName, VarTypeQualifier};

//noinspection ALL
/// Resolves the type of a name-like expression.
/// For bare names, the type comes from their first character, according to
/// the `DEFINT` etc statements.
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

    fn resolve_dim_name_node_to_qualifier<T: VarTypeQualifier>(
        &self,
        dim_name_node: &Locatable<VarName<T>>,
    ) -> TypeQualifier {
        let Locatable {
            element: VarName {
                bare_name,
                var_type,
            },
            ..
        } = dim_name_node;
        self.resolve_dim_name_to_qualifier(bare_name, var_type)
    }

    fn resolve_dim_name_to_qualifier<T: VarTypeQualifier>(
        &self,
        bare_name: &BareName,
        dim_type: &T,
    ) -> TypeQualifier {
        match dim_type.to_qualifier_recursively() {
            Some(q) => q,
            _ => self.resolve(bare_name),
        }
    }
}
