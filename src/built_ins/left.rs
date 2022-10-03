pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNodes;

    pub fn lint(args: &ExpressionNodes) -> Result<(), QErrorNode> {
        if args.len() == 2 {
            args.require_string_argument(0)?;
            args.require_integer_argument(1)
        } else {
            Err(QError::ArgumentCountMismatch).with_err_no_pos()
        }
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::utils::VariantCasts;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let s: &str = interpreter.context()[0].to_str_unchecked();
        let count: usize = interpreter.context()[1].to_non_negative_int()?;
        let left_part: String = s.chars().take(count).collect();
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Left, left_part);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::*;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_happy_flow() {
        assert_prints!(r#"PRINT LEFT$("hay", 0)"#, "");
        assert_prints!(r#"PRINT LEFT$("hay", 1)"#, "h");
        assert_prints!(r#"PRINT LEFT$("hay", 2)"#, "ha");
        assert_prints!(r#"PRINT LEFT$("hay", 3)"#, "hay");
        assert_prints!(r#"PRINT LEFT$("hay", 4)"#, "hay");
    }

    #[test]
    fn test_edge_cases() {
        assert_prints!(r#"PRINT LEFT$("", 1)"#, "");
        assert_interpreter_err!(r#"PRINT LEFT$("a", -1)"#, QError::IllegalFunctionCall, 1, 7);
    }

    #[test]
    fn test_no_args() {
        assert_linter_err!(r#"PRINT LEFT$()"#, QError::FunctionNeedsArguments);
    }

    #[test]
    fn test_one_arg() {
        assert_linter_err!(r#"PRINT LEFT$("oops")"#, QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_three_args() {
        assert_linter_err!(
            r#"PRINT LEFT$("oops", 1, 2)"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT LEFT$(10, 40)"#, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn test_second_arg_string() {
        assert_linter_err!(
            r#"PRINT LEFT$("hello", "world")"#,
            QError::ArgumentTypeMismatch
        );
    }
}
