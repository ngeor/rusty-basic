pub mod linter {
    use crate::common::{
        CanCastTo, Locatable, QError, QErrorNode, ToErrorEnvelopeNoPos, ToLocatableError,
    };
    use crate::parser::{Expression, ExpressionNode, ExpressionType, TypeQualifier, VariableInfo};

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        if args.is_empty() || args.len() > 2 {
            return Err(QError::ArgumentCountMismatch).with_err_no_pos();
        }

        // Can have at one or two arguments. First must be the array name, without parenthesis.
        // Second, optional, is an integer specifying the array dimension >=1 (default is 1).
        let Locatable {
            element: first,
            pos: first_pos,
        } = args.get(0).unwrap();
        if let Expression::Variable(
            _,
            VariableInfo {
                expression_type: ExpressionType::Array(_),
                ..
            },
        ) = first
        {
            if args.len() == 2 {
                if args[1].can_cast_to(TypeQualifier::PercentInteger) {
                    Ok(())
                } else {
                    Err(QError::ArgumentTypeMismatch).with_err_at(&args[1])
                }
            } else {
                Ok(())
            }
        } else {
            Err(QError::ArgumentTypeMismatch).with_err_at(first_pos)
        }
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
                Some((lower, _)) => {
                    interpreter.context_mut().set_built_in_function_result(
                        BuiltInFunction::LBound,
                        Variant::VInteger(*lower),
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
