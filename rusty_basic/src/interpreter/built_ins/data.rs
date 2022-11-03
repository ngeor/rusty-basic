use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;
use rusty_variant::Variant;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let variables: Vec<Variant> = interpreter
        .context()
        .variables()
        .iter()
        .map(Clone::clone)
        .collect();
    for v in variables {
        interpreter.data_segment().push(v);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn data_is_moved_at_the_start_of_the_program() {
        let input = r#"
        READ A
        PRINT A
        DATA 42
        "#;
        assert_prints!(input, "42");
    }
}
