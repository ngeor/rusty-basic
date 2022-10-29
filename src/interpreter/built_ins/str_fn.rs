use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;
use crate::variant::Variant;

macro_rules! str_fmt {
        ($e: expr, $zero: expr) => {
            if $e >= $zero {
                format!(" {}", $e)
            } else {
                format!("{}", $e)
            }
        };
    }

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let v: &Variant = &interpreter.context()[0];
    let result = match v {
        Variant::VSingle(f) => str_fmt!(*f, 0.0),
        Variant::VDouble(f) => str_fmt!(*f, 0.0),
        Variant::VInteger(f) => str_fmt!(*f, 0),
        Variant::VLong(f) => str_fmt!(*f, 0),
        _ => panic!("unexpected arg to STR$"),
    };
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Str, result);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints_exact;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_str_float_positive() {
        let program = r#"PRINT STR$(3.14)"#;
        assert_prints_exact!(program, " 3.14", "");
    }

    #[test]
    fn test_str_float_negative() {
        let program = r#"PRINT STR$(-3.14)"#;
        assert_prints_exact!(program, "-3.14", "");
    }

    #[test]
    fn test_str_integer_positive() {
        let program = r#"PRINT STR$(42)"#;
        assert_prints_exact!(program, " 42", "");
    }

    #[test]
    fn test_str_integer_negative() {
        let program = r#"PRINT STR$(-42)"#;
        assert_prints_exact!(program, "-42", "");
    }
}
