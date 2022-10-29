use crate::common::{QError, ToAsciiBytes};
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;
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

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn prints_expected_value() {
        let program = "PRINT CVD(\"\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}@\")";
        assert_prints!(program, "2");
    }
}
