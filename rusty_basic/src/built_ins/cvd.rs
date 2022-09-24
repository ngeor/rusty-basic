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
    use crate::common::{QError, ToAsciiBytes};
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::bytes_to_f64;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let s = interpreter.context()[0].to_str_unchecked();
        let bytes: Vec<u8> = s.to_ascii_bytes();
        let f = bytes_to_f64(&bytes);
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Cvd, f);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn no_args() {
        assert_linter_err!("PRINT CVD()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT CVD(A$, B$)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_integer() {
        assert_linter_err!("PRINT CVD(10)", QError::ArgumentTypeMismatch);
    }

    #[test]
    fn prints_expected_value() {
        let program = "PRINT CVD(\"\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}@\")";
        assert_prints!(program, "2");
    }
}
