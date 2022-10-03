pub mod linter {
    use crate::common::QErrorNode;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        super::super::lbound::linter::lint(args)
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;
    use crate::variant::Variant;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let v: Variant = interpreter.context()[0].clone();
        let dimension: usize = match interpreter.context().variables().get(1) {
            Some(v) => v.to_positive_int_or(QError::SubscriptOutOfRange)?,
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
                _ => Err(QError::SubscriptOutOfRange),
            },
            _ => Err(QError::ArgumentTypeMismatch),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

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
        assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 15);
    }

    #[test]
    fn test_explicit_lbound_dimension_out_of_range_zero() {
        let input = r#"
        DIM choice$(1 TO 2, 3 TO 4)
        PRINT UBOUND(choice$, 0)
        "#;
        assert_interpreter_err!(input, QError::SubscriptOutOfRange, 3, 15);
    }
}
