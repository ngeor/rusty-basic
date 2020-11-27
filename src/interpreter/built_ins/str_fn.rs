// STR$(numeric-expression) returns a string representation of a number

use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let v: &Variant = interpreter.context().get(0).unwrap();
    let result = match v {
        Variant::VSingle(f) => format!("{}", f),
        Variant::VDouble(f) => format!("{}", f),
        Variant::VInteger(f) => format!("{}", f),
        Variant::VLong(f) => format!("{}", f),
        _ => panic!("unexpected arg to STR$"),
    };
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Str.into(), result.into());
    Ok(())
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
