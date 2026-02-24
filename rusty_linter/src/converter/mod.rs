//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
//! To convert a program, invoke the convert method of the Convertible trait on it.
pub mod common;
mod dim_rules;
mod expr_rules;
mod statement;
