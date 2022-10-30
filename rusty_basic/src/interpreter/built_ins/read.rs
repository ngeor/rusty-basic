use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;
use rusty_parser::TypeQualifier;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    // variables are passed by ref, so we can assign to them
    let len = interpreter.context().variables().len();
    for i in 0..len {
        let target_type =
            TypeQualifier::try_from(interpreter.context().variables().get(i).unwrap())?;
        let data_value = interpreter.data_segment().pop()?;
        let casted_value = data_value.cast(target_type)?;
        interpreter.context_mut()[i] = casted_value;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_interpreter_err;
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use rusty_common::*;

    #[test]
    fn data_read_print() {
        let input = r#"
        DATA "the answer is", 42
        READ A$, B%
        PRINT A$, B%
        "#;
        assert_prints!(input, "the answer is  42");
    }

    #[test]
    fn read_into_property() {
        let input = r#"
        TYPE Card
            Value AS INTEGER
        END TYPE

        DIM C AS Card

        DATA 42
        READ C.Value
        PRINT C.Value
        "#;
        assert_prints!(input, "42");
    }

    #[test]
    fn read_into_array_element() {
        let input = r#"
        DIM A(1 TO 5)
        DATA 1, 5, 9, 6, 7, 3, 2
        READ LO
        READ HI
        FOR I = LO TO HI
            READ A(I)
            PRINT A(I)
        NEXT
        "#;
        assert_prints!(input, "9", "6", "7", "3", "2");
    }

    #[test]
    fn cast_error_at_runtime() {
        let input = r#"
        DATA 42
        READ A$
        "#;
        assert_interpreter_err!(input, QError::TypeMismatch, 3, 9);
    }
}
