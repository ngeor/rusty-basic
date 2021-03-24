pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_string_argument()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let s: &str = interpreter.context()[0].to_str_unchecked();
        let result = s.to_ascii_uppercase();
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::UCase, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_happy_flow() {
        assert_prints!(r#"PRINT UCASE$("hay")"#, "HAY");
        assert_prints!(r#"PRINT UCASE$("WORLD")"#, "WORLD");
        assert_prints!(r#"PRINT UCASE$("Oops")"#, "OOPS");
        assert_prints!(r#"PRINT UCASE$("")"#, "");
    }

    #[test]
    fn test_no_args() {
        assert_linter_err!(
            r#"PRINT UCASE$()"#,
            QError::syntax_error("Cannot have function call without arguments")
        );
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT UCASE$("oops", "oops")"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT UCASE$(10)"#, QError::ArgumentTypeMismatch);
    }
}
