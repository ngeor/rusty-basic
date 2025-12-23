use rusty_parser::specific::QualifiedNamePos;

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
pub type ImplicitVars = Vec<QualifiedNamePos>;
