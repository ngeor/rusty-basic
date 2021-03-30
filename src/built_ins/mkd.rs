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
    use crate::variant::{f64_to_bytes, QBNumberCast};

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let f: f64 = interpreter.context()[0].try_cast()?;
        let bytes = f64_to_bytes(f);
        let s: String = bytes.iter().map(|b| *b as char).collect();
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Mkd, s);
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
        let program = r#"PRINT MKD$(2)"#;
        assert_prints!(program, "\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}@");
    }
}
