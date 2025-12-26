use crate::core::{ConstLookup, LintError};
use rusty_common::Positioned;
use rusty_parser::{Expression, TypeQualifier};
use rusty_variant::{Variant, MAX_INTEGER};

pub trait ValidateStringLength<E> {
    fn validate_string_length(&self, const_lookup: &impl ConstLookup) -> Result<u16, E>;
}

impl<T, E> ValidateStringLength<Positioned<E>> for Positioned<T>
where
    T: ValidateStringLength<E>,
{
    fn validate_string_length(
        &self,
        const_lookup: &impl ConstLookup,
    ) -> Result<u16, Positioned<E>> {
        let Positioned { element, pos } = self;
        element
            .validate_string_length(const_lookup)
            .map_err(|e| Positioned::new(e, *pos))
    }
}

impl ValidateStringLength<LintError> for Expression {
    fn validate_string_length(&self, const_lookup: &impl ConstLookup) -> Result<u16, LintError> {
        match self {
            Self::IntegerLiteral(i) => {
                if (1..=MAX_INTEGER).contains(i) {
                    Ok(*i as u16)
                } else {
                    Err(LintError::InvalidConstant)
                }
            }
            Self::Variable(name, _) => {
                if let Some(qualifier) = name.qualifier() {
                    if qualifier != TypeQualifier::PercentInteger {
                        return Err(LintError::InvalidConstant);
                    }
                }

                if let Some(Variant::VInteger(i)) =
                    const_lookup.get_resolved_constant(name.bare_name())
                {
                    if (1..=MAX_INTEGER).contains(i) {
                        return Ok(*i as u16);
                    }
                }

                Err(LintError::InvalidConstant)
            }
            _ => Err(LintError::InvalidConstant),
        }
    }
}
