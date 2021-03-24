pub mod linter {
    use crate::common::{QError, QErrorNode, ToErrorEnvelopeNoPos};
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
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
        let right_part: String = if s.len() > count {
            s.chars().skip(s.len() - count).collect()
        } else {
            s.to_owned()
        };
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Right, right_part);
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
        assert_prints!(r#"PRINT RIGHT$("hay", 0)"#, "");
        assert_prints!(r#"PRINT RIGHT$("hay", 1)"#, "y");
        assert_prints!(r#"PRINT RIGHT$("hay", 2)"#, "ay");
        assert_prints!(r#"PRINT RIGHT$("hay", 3)"#, "hay");
        assert_prints!(r#"PRINT RIGHT$("hay", 4)"#, "hay");
    }

    #[test]
    fn test_edge_cases() {
        assert_prints!(r#"PRINT RIGHT$("", 1)"#, "");
        assert_interpreter_err!(
            r#"PRINT RIGHT$("a", -1)"#,
            QError::IllegalFunctionCall,
            1,
            7
        );
    }

    #[test]
    fn test_no_args() {
        assert_linter_err!(
            r#"PRINT RIGHT$()"#,
            QError::syntax_error("Cannot have function call without arguments")
        );
    }

    #[test]
    fn test_one_arg() {
        assert_linter_err!(r#"PRINT RIGHT$("oops")"#, QError::ArgumentCountMismatch);
    }

    #[test]
    fn test_three_args() {
        assert_linter_err!(
            r#"PRINT RIGHT$("oops", 1, 2)"#,
            QError::ArgumentCountMismatch
        );
    }

    #[test]
    fn test_first_arg_integer() {
        assert_linter_err!(r#"PRINT RIGHT$(10, 40)"#, QError::ArgumentTypeMismatch);
    }

    #[test]
    fn test_second_arg_string() {
        assert_linter_err!(
            r#"PRINT RIGHT$("hello", "world")"#,
            QError::ArgumentTypeMismatch
        );
    }
}
