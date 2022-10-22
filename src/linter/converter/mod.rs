//! Converter is the main logic of the linter, where most validation takes place,
//! as well as resolving variable types.
mod converter;
mod dim_rules;
mod expr_rules;
mod function_implementation;
mod names;
mod statement;
mod sub_implementation;
mod traits;

pub use self::converter::convert;
