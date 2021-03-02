use super::*;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let v: Variant = interpreter.context()[0].clone();
    let dimension: i32 = match interpreter.context().variables().get(1) {
        Some(v) => v
            .clone()
            .cast(TypeQualifier::PercentInteger)
            .and_then(|v| i32::try_from(v))
            .with_err_no_pos()?,
        _ => 1,
    };
    if dimension <= 0 {
        Err(QError::SubscriptOutOfRange).with_err_no_pos()
    } else {
        match v {
            Variant::VArray(a) => match a.get_dimensions((dimension - 1) as usize) {
                Some((lower, _)) => {
                    interpreter.context_mut().set_built_in_function_result(
                        BuiltInFunction::LBound,
                        Variant::VInteger(*lower),
                    );
                    Ok(())
                }
                _ => Err(QError::SubscriptOutOfRange).with_err_no_pos(),
            },
            _ => Err(QError::ArgumentTypeMismatch).with_err_no_pos(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_interpreter_err;
    use crate::assert_prints;

    #[test]
    fn test_explicit_lbound() {
        let input = r#"
        DIM choice$(1 TO 3)
        PRINT LBOUND(choice$)
        "#;
        assert_prints!(input, "1");
    }

    #[test]
    fn test_implicit_lbound() {
        let input = r#"
        DIM choice$(3)
        PRINT LBOUND(choice$)
        "#;
        assert_prints!(input, "0");
    }

    #[test]
    fn test_explicit_lbound_explicit_primary_dimension() {
        let input = r#"
        DIM choice$(1 TO 2)
        PRINT LBOUND(choice$, 1)
        "#;
        assert_prints!(input, "1");
    }

    #[test]
    fn test_explicit_lbound_secondary_dimension() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT LBOUND(choice$, 2)
        "#;
        assert_prints!(input, "3");
    }

    #[test]
    fn test_explicit_lbound_dimension_out_of_range() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT LBOUND(choice$, 3)
        "#;
        assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 15);
    }

    #[test]
    fn test_explicit_lbound_dimension_out_of_range_zero() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT LBOUND(choice$, 0)
        "#;
        assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 15);
    }
}
