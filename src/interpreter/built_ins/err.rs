use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::BuiltInFunction;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
    let error_code: i32 = interpreter.get_last_error_code().unwrap_or_default();
    interpreter
        .context_mut()
        .set_built_in_function_result(BuiltInFunction::Err, error_code);
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::assert_prints;
    use crate::interpreter::interpreter_trait::InterpreterTrait;

    #[test]
    fn test_err() {
        let input = r#"
        ON ERROR GOTO ErrTrap
        OPEN "whatever.txt" FOR INPUT AS #1
        CLOSE
        END

        ErrTrap:
            SELECT CASE ERR
            CASE 53
                PRINT "File not found"
            CASE ELSE
                PRINT "oops"
            END SELECT
            RESUME NEXT
        "#;
        assert_prints!(input, "File not found");
    }

    #[test]
    fn test_resume_clears_err() {
        let input = r#"
        ON ERROR GOTO ErrTrap
        OPEN "whatever.txt" FOR INPUT AS #1
        CLOSE
        PRINT ERR
        END

        ErrTrap:
            PRINT ERR
            RESUME NEXT
        "#;
        assert_prints!(input, "53", "0");
    }
}
