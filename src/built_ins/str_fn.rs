// STR$(numeric-expression) returns a string representation of a number
// TODO support hexadecimal literals &H10
use super::BuiltInRun;
use crate::common::*;
use crate::interpreter::{Interpreter, Stdlib};
use crate::variant::Variant;

pub struct StrFn {}

impl BuiltInRun for StrFn {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VSingle(f) => Variant::VString(format!("{}", f)),
            Variant::VDouble(f) => Variant::VString(format!("{}", f)),
            Variant::VInteger(f) => Variant::VString(format!("{}", f)),
            Variant::VLong(f) => Variant::VString(format!("{}", f)),
            _ => panic!("unexpected arg to STR$"),
        };
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;

    #[test]
    fn test_str_float() {
        let program = r#"PRINT STR$(3.14)"#;
        assert_prints!(program, "3.14");
    }
}
