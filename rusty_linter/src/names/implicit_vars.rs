use rusty_common::Positioned;
use rusty_parser::Name;

/// Alias for the implicit variables collected during evaluating something.
/// e.g. `INPUT N` is a statement implicitly defining variable `N`.
/// The collected implicit variable names are resolved and qualified (i.e. not bare).
pub type ImplicitVars = Vec<Positioned<Name>>;
