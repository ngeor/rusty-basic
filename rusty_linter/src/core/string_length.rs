use rusty_common::Positioned;
use rusty_parser::{AsBareName, Expression, TypeQualifier};
use rusty_variant::{MAX_INTEGER, Variant};

use crate::core::{ConstLookup, LintError};

pub trait ValidateStringLength<E, C: ConstLookup + ?Sized> {
    fn validate_string_length(&self, const_lookup: &C) -> Result<u16, E>;
}

impl<T, E, C> ValidateStringLength<Positioned<E>, C> for Positioned<T>
where
    T: ValidateStringLength<E, C>,
    C: ConstLookup + ?Sized,
{
    fn validate_string_length(&self, const_lookup: &C) -> Result<u16, Positioned<E>> {
        let Positioned { element, pos } = self;
        element
            .validate_string_length(const_lookup)
            .map_err(|e| Positioned::new(e, *pos))
    }
}

impl<C: ConstLookup + ?Sized> ValidateStringLength<LintError, C> for Expression {
    fn validate_string_length(&self, const_lookup: &C) -> Result<u16, LintError> {
        match self {
            Self::IntegerLiteral(i) => {
                if (1..=MAX_INTEGER).contains(i) {
                    Ok(*i as u16)
                } else {
                    Err(LintError::InvalidConstant)
                }
            }
            Self::Variable(name, _) => {
                if let Some(qualifier) = name.qualifier()
                    && qualifier != TypeQualifier::PercentInteger
                {
                    return Err(LintError::InvalidConstant);
                }

                if let Some(Variant::VInteger(i)) =
                    const_lookup.get_const_value(name.as_bare_name())
                    && (1..=MAX_INTEGER).contains(i)
                {
                    return Ok(*i as u16);
                }

                Err(LintError::InvalidConstant)
            }
            _ => Err(LintError::InvalidConstant),
        }
    }
}
