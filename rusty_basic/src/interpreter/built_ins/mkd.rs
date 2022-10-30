use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::{QError, ToAsciiString};
use rusty_parser::variant::{f64_to_bytes, QBNumberCast};
use rusty_parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let f: f64 = interpreter.context()[0].try_cast()?;
    let bytes = f64_to_bytes(f);
    let s: String = bytes.to_ascii_string();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Mkd, s);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn prints_expected_value() {
        let program = r#"PRINT MKD$(2)"#;
        assert_prints!(program, "\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}\u{0}@");
    }
}
