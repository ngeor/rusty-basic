use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;
use crate::variant::QBNumberCast;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let len: i32 = interpreter.context()[0].try_cast()?;
    let mut s: String = String::new();
    for _ in 0..len {
        s.push(' ');
    }
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Space, s);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints_exact;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test() {
        assert_prints_exact!("PRINT SPACE$(4)", "    ", "");
    }
}
