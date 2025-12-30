use rusty_linter::QBNumberCast;
use rusty_parser::BuiltInFunction;

use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let i: i32 = interpreter.context()[0].try_cast()?;
    let mut s: String = String::new();
    s.push((i as u8) as char);
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Chr, s);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_chr() {
        assert_prints!("PRINT CHR$(33)", "!");
    }
}
