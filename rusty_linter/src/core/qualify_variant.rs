use rusty_parser::TypeQualifier;
use rusty_variant::Variant;

use crate::core::LintError;

pub fn qualifier_of_variant(variant: &Variant) -> Result<TypeQualifier, LintError> {
    match variant {
        Variant::VSingle(_) => Ok(TypeQualifier::BangSingle),
        Variant::VDouble(_) => Ok(TypeQualifier::HashDouble),
        Variant::VString(_) => Ok(TypeQualifier::DollarString),
        Variant::VInteger(_) => Ok(TypeQualifier::PercentInteger),
        Variant::VLong(_) => Ok(TypeQualifier::AmpersandLong),
        _ => Err(LintError::InvalidConstant),
    }
}

pub fn qualifier_of_const_variant(variant: &Variant) -> TypeQualifier {
    qualifier_of_variant(variant).expect("Invalid const variant")
}
