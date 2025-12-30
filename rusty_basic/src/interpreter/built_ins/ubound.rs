use rusty_parser::BuiltInFunction;
use rusty_variant::Variant;

use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::variant_casts::VariantCasts;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let v: Variant = interpreter.context()[0].clone();
    let dimension: usize = match interpreter.context().variables().get(1) {
        Some(v) => v.to_positive_int_or(RuntimeError::SubscriptOutOfRange)?,
        _ => 1,
    };
    match v {
        Variant::VArray(a) => match a.get_dimension_bounds(dimension - 1) {
            Some((_, upper)) => {
                interpreter.context_mut().set_built_in_function_result(
                    BuiltInFunction::UBound,
                    Variant::VInteger(*upper),
                );
                Ok(())
            }
            _ => Err(RuntimeError::SubscriptOutOfRange),
        },
        _ => Err(RuntimeError::TypeMismatch),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::{assert_interpreter_err, assert_prints};

    #[test]
    fn test_explicit_lbound() {
        let input = r#"
        DIM choice$(1 TO 3)
        PRINT UBOUND(choice$)
        "#;
        assert_prints!(input, "3");
    }

    #[test]
    fn test_implicit_lbound() {
        let input = r#"
        DIM choice$(4)
        PRINT UBOUND(choice$)
        "#;
        assert_prints!(input, "4");
    }

    #[test]
    fn test_explicit_lbound_explicit_primary_dimension() {
        let input = r#"
        DIM choice$(1 TO 2)
        PRINT UBOUND(choice$, 1)
        "#;
        assert_prints!(input, "2");
    }

    #[test]
    fn test_explicit_lbound_secondary_dimension() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT UBOUND(choice$, 2)
        "#;
        assert_prints!(input, "4");
    }

    #[test]
    fn test_explicit_lbound_dimension_out_of_range() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT UBOUND(choice$, 3)
        "#;
        assert_interpreter_err!(input, RuntimeError::SubscriptOutOfRange, 3, 15);
    }

    #[test]
    fn test_explicit_lbound_dimension_out_of_range_zero() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT UBOUND(choice$, 0)
        "#;
        assert_interpreter_err!(input, RuntimeError::SubscriptOutOfRange, 3, 15);
    }
}
