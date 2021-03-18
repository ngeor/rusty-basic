pub mod linter {
    use crate::common::QErrorNode;
    use crate::linter::arg_validation::ArgValidation;
    use crate::parser::ExpressionNode;

    pub fn lint(args: &Vec<ExpressionNode>) -> Result<(), QErrorNode> {
        args.require_one_numeric_argument()
    }
}

pub mod interpreter {
    use crate::built_ins::BuiltInFunction;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::variant::Variant;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let v: &Variant = &interpreter.context()[0];
        let result = match v {
            Variant::VSingle(f) => format!("{}", f),
            Variant::VDouble(f) => format!("{}", f),
            Variant::VInteger(f) => format!("{}", f),
            Variant::VLong(f) => format!("{}", f),
            _ => panic!("unexpected arg to STR$"),
        };
        interpreter
            .context_mut()
            .set_built_in_function_result(BuiltInFunction::Str, result);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_str_float() {
        let program = r#"PRINT STR$(3.14)"#;
        assert_prints!(program, "3.14");
    }
}
