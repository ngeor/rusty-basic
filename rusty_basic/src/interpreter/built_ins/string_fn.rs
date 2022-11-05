use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::utils::VariantCasts;
use rusty_common::*;
use rusty_linter::QBNumberCast;
use rusty_parser::BuiltInFunction;
use rusty_variant::Variant;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let count: usize = interpreter.context()[0].to_non_negative_int()?;
    let v = &interpreter.context()[1];
    let s = run_with_variant(count, v)?;
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::String, s);
    Ok(())
}

fn run_with_variant(count: usize, v: &Variant) -> Result<String, QError> {
    if let Variant::VString(s) = v {
        run_with_string_argument(count, s)
    } else {
        let ascii: i32 = v.try_cast()?;
        run_with_ascii_code_argument(count, ascii)
    }
}

fn run_with_string_argument(count: usize, s: &str) -> Result<String, QError> {
    let first_char = s.chars().next().ok_or(QError::IllegalFunctionCall)?;
    run_with_char(count, first_char)
}

fn run_with_ascii_code_argument(count: usize, ascii: i32) -> Result<String, QError> {
    if (0..=255).contains(&ascii) {
        let u: u8 = ascii as u8;
        run_with_char(count, u as char)
    } else {
        Err(QError::IllegalFunctionCall)
    }
}

fn run_with_char(count: usize, ch: char) -> Result<String, QError> {
    Ok(std::iter::repeat(ch).take(count).collect())
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use rusty_common::*;

    #[test]
    fn string_with_ascii_code() {
        assert_prints!("PRINT STRING$(3, 33)", "!!!");
    }

    #[test]
    fn string_with_string_argument() {
        assert_prints!(r#"PRINT STRING$(4, "hello")"#, "hhhh");
    }

    #[test]
    fn string_with_empty_string_argument() {
        assert_interpreter_err!(r#"PRINT STRING$(5, "")"#, QError::IllegalFunctionCall, 1, 7);
    }

    #[test]
    fn string_with_zero_count() {
        assert_prints!(r#"PRINT STRING$(0, "hello")"#, "");
    }

    #[test]
    fn string_with_negative_count() {
        assert_interpreter_err!(
            r#"PRINT STRING$(-1, "hello")"#,
            QError::IllegalFunctionCall,
            1,
            7
        );
    }

    #[test]
    fn string_with_negative_ascii_code() {
        assert_interpreter_err!("PRINT STRING$(10, -1)", QError::IllegalFunctionCall, 1, 7);
    }

    #[test]
    fn string_with_too_big_ascii_code() {
        assert_interpreter_err!("PRINT STRING$(10, 256)", QError::IllegalFunctionCall, 1, 7);
    }
}
