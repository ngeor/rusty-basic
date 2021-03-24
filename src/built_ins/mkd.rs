pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_double_argument()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::Variant;

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
        assert_linter_err!("PRINT MKD$()", QError::FunctionNeedsArguments);
    }

    #[test]
    fn two_args() {
        assert_linter_err!("PRINT MKD$(A#, B#)", QError::ArgumentCountMismatch);
    }

    #[test]
    fn first_arg_string() {
        assert_linter_err!("PRINT MKD$(\"10\")", QError::ArgumentTypeMismatch);
    }

    #[test]
    fn prints_expected_value() {
        let program = r#"PRINT MKD$(3.14)"#;
        assert_prints!(program, "fix me");
    }
}
