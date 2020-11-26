// CHR$(ascii-code%) returns the text representation of the given ascii code
use super::*;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
    let i: i32 = interpreter
        .context()
        .get(0)
        .unwrap()
        .try_into()
        .with_err_no_pos()?;
    let mut s: String = String::new();
    s.push((i as u8) as char);
    interpreter
        .context_mut()
        .set_variable(BuiltInFunction::Chr.into(), s.into());
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::assert_prints;
    use crate::common::QError;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    #[test]
    fn test_chr() {
        assert_prints!("PRINT CHR$(33)", "!");
        assert_linter_err!("PRINT CHR$(33, 34)", QError::ArgumentCountMismatch, 1, 7);
        assert_linter_err!(r#"PRINT CHR$("33")"#, QError::ArgumentTypeMismatch, 1, 12);
    }
}
