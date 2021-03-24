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
        todo!()
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
        let program = r#"PRINT CVD("12345678")"#;
        assert_prints!(program, "fix me");
    }
}
