pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        args.require_one_string_argument()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let s: &str = interpreter.context()[0].to_str_unchecked();
        let result = s.to_ascii_lowercase();
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::LCase, result);
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
        assert_prints!(r#"PRINT LCASE$("hay")"#, "hay");
        assert_prints!(r#"PRINT LCASE$("WORLD")"#, "world");
        assert_prints!(r#"PRINT LCASE$("Oops")"#, "oops");
        assert_prints!(r#"PRINT LCASE$("")"#, "");
    }

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LCASE$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_two_arg() {
        assert_linter_err!(
            r#"PRINT LCASE$("oops", "oops")"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LCASE$(10)"#, QError::ArgumentTypeMismatch);
    }
}
