mod arg_validation;
mod built_ins;
mod casting;
mod const_value_resolver;
mod converter;
mod linter;
mod post_linter;
mod pre_linter;
mod qb_casting;
mod traits;
mod type_resolver;
mod type_resolver_impl;
mod types;

#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

pub use self::linter::lint;
pub use self::qb_casting::{CastVariant, QBNumberCast};
pub use self::traits::*;
pub use self::types::*;
use rusty_common::QError;
use rusty_parser::TypeQualifier;
use rusty_variant::Variant;

// TODO move to a module
pub fn qualifier_of_variant(variant: &Variant) -> Result<TypeQualifier, QError> {
    match variant {
        Variant::VSingle(_) => Ok(TypeQualifier::BangSingle),
        Variant::VDouble(_) => Ok(TypeQualifier::HashDouble),
        Variant::VString(_) => Ok(TypeQualifier::DollarString),
        Variant::VInteger(_) => Ok(TypeQualifier::PercentInteger),
        Variant::VLong(_) => Ok(TypeQualifier::AmpersandLong),
        _ => Err(QError::InvalidConstant),
    }
}

fn qualifier_of_const_variant(variant: &Variant) -> TypeQualifier {
    qualifier_of_variant(variant).expect("Invalid const variant")
}
