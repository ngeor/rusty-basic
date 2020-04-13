use super::TypeQualifier;
use crate::common::CaseInsensitiveString;

pub trait TypeResolver {
    fn resolve(&self, name: &CaseInsensitiveString) -> TypeQualifier;
}

pub trait HasBareName {
    fn bare_name(&self) -> &CaseInsensitiveString;
    fn bare_name_into(self) -> CaseInsensitiveString;
}

pub trait HasQualifier {
    fn qualifier(&self) -> TypeQualifier;
}

pub trait ResolvesQualifier {
    fn qualifier(&self, resolver: &dyn TypeResolver) -> TypeQualifier;
}
