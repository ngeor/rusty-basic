use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::Input;
use crate::RuntimeError;
use rusty_linter::qualifier_of_variant;
use rusty_parser::{FileHandle, TypeQualifier};
use rusty_variant::Variant;
use std::convert::TryFrom;

pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), RuntimeError> {
    let mut file_handle: FileHandle = FileHandle::default();
    let mut has_file_handle = false;
    for index in 0..interpreter.context().variables().len() {
        let v: &Variant = &interpreter.context()[index];
        match v {
            Variant::VInteger(f) => {
                if index == 0 {
                    has_file_handle = *f == 1;
                } else if index == 1 {
                    if has_file_handle {
                        file_handle = FileHandle::try_from(*f)
                            .map_err(|_| RuntimeError::BadFileNameOrNumber)?;
                    } else {
                        // input integer variable
                        do_input_one_var(interpreter, index, file_handle)?;
                    }
                } else {
                    // input integer variable
                    do_input_one_var(interpreter, index, file_handle)?;
                }
            }
            _ => {
                do_input_one_var(interpreter, index, file_handle)?;
            }
        }
    }
    Ok(())
}

fn do_input_one_var<S: InterpreterTrait>(
    interpreter: &mut S,
    index: usize,
    file_handle: FileHandle,
) -> Result<(), RuntimeError> {
    let raw_input: String = raw_input(interpreter, file_handle)?;
    let q: TypeQualifier = qualifier(interpreter, index)?;
    let new_value: Variant = match q {
        TypeQualifier::BangSingle => Variant::from(parse_single_input(raw_input)?),
        TypeQualifier::DollarString => Variant::from(raw_input),
        TypeQualifier::PercentInteger => Variant::from(parse_int_input(raw_input)?),
        _ => todo!("INPUT type {} not supported yet", q),
    };
    interpreter.context_mut()[index] = new_value;
    Ok(())
}

fn raw_input<S: InterpreterTrait>(
    interpreter: &mut S,
    file_handle: FileHandle,
) -> Result<String, RuntimeError> {
    if file_handle.is_valid() {
        let file_input = interpreter
            .file_manager()
            .try_get_file_info_input(&file_handle)?;
        file_input.input().map_err(RuntimeError::from)
    } else {
        interpreter.stdin().input().map_err(RuntimeError::from)
    }
}

fn qualifier<S: InterpreterTrait>(
    interpreter: &S,
    index: usize,
) -> Result<TypeQualifier, RuntimeError> {
    qualifier_of_variant(&interpreter.context()[index]).map_err(RuntimeError::from)
}

fn parse_single_input(s: String) -> Result<f32, RuntimeError> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| RuntimeError::Other(format!("Could not parse {} as float: {}", s, e)))
    }
}

fn parse_int_input(s: String) -> Result<i32, RuntimeError> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| RuntimeError::Other(format!("Could not parse {} as int: {}", s, e)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::assert_has_variable;
    use crate::assert_interpreter_err;
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::{
        interpret, interpret_with_raw_input, MockInterpreterTrait,
    };
    use rusty_variant::Variant;

    fn assert_input<T>(
        raw_input: &str,
        variable_literal: &str,
        qualified_variable: &str,
        expected_value: T,
    ) where
        Variant: From<T>,
    {
        let input = format!("INPUT {}", variable_literal);
        let interpreter = interpret_with_raw_input(&input, raw_input);
        assert_has_variable!(interpreter, qualified_variable, expected_value);
    }

    mod unqualified_var {
        use super::*;

        #[test]
        fn test_input_empty() {
            assert_input("\r\n", "N", "N!", 0.0_f32);
        }

        #[test]
        fn test_input_zero() {
            assert_input("0", "N", "N!", 0.0_f32);
        }

        #[test]
        fn test_input_single() {
            assert_input("1.1", "N", "N!", 1.1_f32);
        }

        #[test]
        fn test_input_negative() {
            assert_input("-1.2345", "N", "N!", -1.2345_f32);
        }

        #[test]
        fn test_input_explicit_positive() {
            assert_input("+9.14", "N", "N!", 9.14_f32);
        }

        #[test]
        fn test_space_is_trimmed() {
            assert_input("  42  ", "N", "N!", 42.0_f32);
        }
    }

    mod string_var {
        use super::*;
        use crate::interpreter::interpreter_trait::InterpreterTrait;

        #[test]
        fn test_input_one_variable_hello_no_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello");
            assert_eq!(interpreter.stdout().output(), "hello.");
        }

        #[test]
        fn test_input_one_variable_hello_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello\r\n");
            assert_eq!(interpreter.stdout().output_exact(), "hello.\r\n");
        }

        #[test]
        fn test_input_two_variables_hello_world_comma_no_cr_lf() {
            let input = r#"
            INPUT A$
            PRINT A$ + "."
            INPUT A$
            PRINT A$ + "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "hello, world");
            assert_eq!(interpreter.stdout().output_exact(), "hello.\r\nworld.\r\n");
        }

        #[test]
        fn test_space_is_trimmed() {
            let input = r#"
            INPUT A$
            PRINT A$ ; "."
            "#;
            let mut interpreter = interpret_with_raw_input(input, "  hello  ");
            assert_eq!(interpreter.stdout().output(), "hello.");
        }
    }

    mod int_var {
        use super::*;

        #[test]
        fn test_input_42() {
            assert_input("42", "A%", "A%", 42);
        }
    }

    #[test]
    fn test_input_dim_extended_builtin() {
        let input = "
        DIM X AS INTEGER
        INPUT X
        PRINT X
        ";
        let mut interpreter = interpret_with_raw_input(input, "42");
        assert_eq!(interpreter.stdout().output(), "42");
    }

    #[test]
    fn test_input_dim_user_defined() {
        let input = "
        TYPE Card
            Value AS INTEGER
            Suit AS STRING * 9
        END TYPE

        DIM X AS Card
        INPUT X.Value
        INPUT X.Suit
        PRINT X.Value
        PRINT X.Suit
        ";
        let mut interpreter = interpret_with_raw_input(input, "2, diamonds are forever");
        assert_eq!(interpreter.stdout().output_exact(), " 2 \r\ndiamonds \r\n");
    }

    #[test]
    fn test_input_inside_sub() {
        let input = "
        TYPE Card
            Value AS INTEGER
        END TYPE

        DIM X AS Card
        Test X.Value
        PRINT X.Value

        SUB Test(X%)
            INPUT X%
        END SUB
        ";
        let mut interpreter = interpret_with_raw_input(input, "42");
        assert_eq!(interpreter.stdout().output(), "42");
    }

    #[test]
    fn test_input_assign_to_function_directly() {
        let input = "
        X = Test
        PRINT X

        FUNCTION Test
            INPUT Test
        END FUNCTION
        ";
        let mut interpreter = interpret_with_raw_input(input, "3.14");
        assert_eq!(interpreter.stdout().output(), "3.14");
    }

    #[test]
    fn test_input_string_from_file() {
        // arrange
        std::fs::write("test1.txt", "Hello, world").unwrap();

        // act
        let input = r#"
        OPEN "test1.txt" FOR INPUT AS #1
        OPEN "test2.txt" FOR OUTPUT AS #2
        INPUT #1, A$
        PRINT #2, A$
        INPUT #1, A$
        PRINT #2, A$
        CLOSE
        "#;
        interpret(input);

        // assert
        let bytes = std::fs::read("test2.txt").expect("Should have created output file");
        let s: String = String::from_utf8(bytes).expect("Should be valid utf-8");
        std::fs::remove_file("test1.txt").unwrap_or_default();
        std::fs::remove_file("test2.txt").unwrap_or_default();
        assert_eq!(s, "Hello\r\nworld\r\n");
    }

    #[test]
    fn test_input_string_from_file_eof() {
        std::fs::write("test_input_string_from_file_eof.txt", "Hello, world").unwrap();

        let input = r#"
        OPEN "test_input_string_from_file_eof.txt" FOR INPUT AS #1
        INPUT #1, A$
        INPUT #1, A$
        INPUT #1, A$ ' should EOF here
        CLOSE
        "#;

        assert_interpreter_err!(input, RuntimeError::InputPastEndOfFile, 5, 9);
        std::fs::remove_file("test_input_string_from_file_eof.txt").unwrap_or_default();
    }
}
