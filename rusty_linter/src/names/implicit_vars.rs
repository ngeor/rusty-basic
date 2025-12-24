use rusty_common::Positioned;
use rusty_parser::QualifiedName;

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
pub type ImplicitVars = Vec<Positioned<QualifiedName>>;
