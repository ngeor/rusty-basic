use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{FileAccess, FileHandle, FileMode, QError, QErrorNode, ToErrorEnvelopeNoPos};
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::print;
use crate::interpreter::stdlib::Stdlib;
use crate::linter::{ElementType, UserDefinedType, UserDefinedTypes};
use crate::parser::TypeQualifier;
use crate::variant::{Variant, MAX_INTEGER, MAX_LONG};
use std::convert::TryInto;

pub fn run_function<S: InterpreterTrait>(
    f: &BuiltInFunction,
    interpreter: &mut S,
) -> Result<(), QErrorNode> {
    match f {
        BuiltInFunction::Chr => chr::run(interpreter),
        BuiltInFunction::Environ => environ_fn::run(interpreter),
        BuiltInFunction::Eof => eof::run(interpreter).with_err_no_pos(),
        BuiltInFunction::InStr => instr::run(interpreter),
        BuiltInFunction::LBound => todo!(),
        BuiltInFunction::Len => len::run(interpreter),
        BuiltInFunction::Mid => mid::run(interpreter),
        BuiltInFunction::Str => str_fn::run(interpreter),
        BuiltInFunction::UBound => todo!(),
        BuiltInFunction::Val => val::run(interpreter),
    }
}

pub fn run_sub<S: InterpreterTrait>(s: &BuiltInSub, interpreter: &mut S) -> Result<(), QErrorNode> {
    match s {
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Input => input::run(interpreter).with_err_no_pos(),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter).with_err_no_pos(),
        BuiltInSub::LPrint => todo!("LPT1 printing not implemented yet"),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
        BuiltInSub::Print => print::run(interpreter).with_err_no_pos(),
        BuiltInSub::System => system::run(interpreter),
    }
}

mod chr {
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
}

mod close {
    // CLOSE
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let file_handles: Vec<FileHandle> = (0..interpreter.context().parameter_count())
            .map(|idx| interpreter.context().get(idx).unwrap())
            .map(|v| v.try_into())
            .collect::<Result<Vec<FileHandle>, QError>>()
            .with_err_no_pos()?;
        if file_handles.is_empty() {
            interpreter.file_manager().close_all();
        } else {
            for file_handle in file_handles {
                interpreter.file_manager().close(&file_handle);
            }
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::interpreter::test_utils::*;

        #[test]
        fn test_close_not_opened_file_is_allowed() {
            interpret("CLOSE 1");
        }

        #[test]
        fn test_close_allows_to_open_again() {
            let input = r#"
            OPEN "a.txt" FOR OUTPUT AS #1
            CLOSE #1
            OPEN "a.txt" FOR OUTPUT AS #1
            CLOSE #1
            "#;
            interpret(input);
            std::fs::remove_file("a.txt").unwrap_or(());
        }

        #[test]
        fn test_close_without_args_closes_all_files() {
            let input = r#"
            OPEN "a.txt" FOR OUTPUT AS #1
            OPEN "b.txt" FOR OUTPUT AS #2
            CLOSE
            OPEN "a.txt" FOR OUTPUT AS #1
            OPEN "b.txt" FOR OUTPUT AS #2
            CLOSE
            "#;
            interpret(input);
            std::fs::remove_file("a.txt").unwrap_or(());
            std::fs::remove_file("b.txt").unwrap_or(());
        }
    }
}

mod environ_fn {
    // ENVIRON$ (env-variable$) -> returns the variable
    // ENVIRON$ (n%) -> returns the nth variable (TODO support this)
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let env_var_name: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let result = interpreter.stdlib().get_env_var(env_var_name);
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Environ.into(), result.into());
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_has_variable;
        use crate::assert_linter_err;
        use crate::common::QError;
        use crate::interpreter::interpreter_trait::InterpreterTrait;
        use crate::interpreter::stdlib::Stdlib;
        use crate::interpreter::test_utils::*;

        #[test]
        fn test_function_call_environ() {
            let program = r#"
            X$ = ENVIRON$("abc")
            Y$ = ENVIRON$("def")
            "#;
            let interpreter = interpret_with_env(program, |x| {
                x.stdlib_mut()
                    .set_env_var("abc".to_string(), "foo".to_string())
            });
            assert_has_variable!(interpreter, "X$", "foo");
            assert_has_variable!(interpreter, "Y$", "");
        }

        #[test]
        fn test_function_call_environ_two_args_linter_err() {
            assert_linter_err!(
                r#"X$ = ENVIRON$("hi", "bye")"#,
                QError::ArgumentCountMismatch,
                1,
                6
            );
        }

        #[test]
        fn test_function_call_environ_numeric_arg_linter_err() {
            assert_linter_err!("X$ = ENVIRON$(42)", QError::ArgumentTypeMismatch, 1, 15);
        }
    }
}

mod environ_sub {
    // ENVIRON str-expr$ -> sets the variable.
    // Parameter must be in the form of name=value or name value (TODO support the latter)
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let s: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let parts: Vec<&str> = s.split("=").collect();
        if parts.len() != 2 {
            Err(QError::from("Invalid expression. Must be name=value.")).with_err_no_pos()
        } else {
            let name = parts[0].to_string();
            let value = parts[1].to_string();
            interpreter.stdlib_mut().set_env_var(name, value);
            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::interpreter::interpreter_trait::InterpreterTrait;
        use crate::interpreter::stdlib::Stdlib;
        use crate::interpreter::test_utils::interpret;

        #[test]
        fn test_sub_call_environ() {
            let program = r#"
            ENVIRON "FOO=BAR"
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib().get_env_var(&"FOO".to_string()), "BAR");
        }

        #[test]
        fn test_sub_call_environ_by_ref() {
            let program = r#"
            A$ = "FOO1=BAR2"
            ENVIRON A$
            "#;
            let interpreter = interpret(program);
            assert_eq!(
                interpreter.stdlib().get_env_var(&"FOO1".to_string()),
                "BAR2"
            );
        }
    }
}

mod eof {
    // EOF(file-number%) -> checks if the end of file has been reached
    use super::*;
    use crate::interpreter::input::Input;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let file_handle: FileHandle = interpreter.context().get(0).unwrap().try_into()?;
        let file_input = interpreter
            .file_manager()
            .try_get_file_info_input_mut(&file_handle)?;
        let is_eof: bool = file_input.eof()?;
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Eof.into(), is_eof.into());
        Ok(())
    }
}

mod input {
    // INPUT [;] ["prompt"{; | ,}] variable-list
    // INPUT #file-number%, variable-list
    //
    // prompt - An optional string that is displayed before the user
    // enters data. A semicolon after promp appends a question mark to the
    // prompt string.
    //
    // variable names can consist of up to 40 characters and must begin
    // with a letter. Valid characters are a-z, 0-9 and period (.).
    //
    // A semicolon immediately after INPUT keeps the cursor on the same line
    // after the user presses the Enter key.

    use super::*;
    use crate::interpreter::input::Input;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_file_handle = false;
        for idx in 0..interpreter.context().parameter_count() {
            let v = interpreter.context().get(idx).unwrap();
            match v {
                Variant::VInteger(f) => {
                    if idx == 0 {
                        has_file_handle = *f == 1;
                    } else if idx == 1 {
                        if has_file_handle {
                            file_handle = FileHandle::try_from(*f)?;
                        } else {
                            // input integer variable
                            do_input_one_var(interpreter, idx, file_handle)?;
                        }
                    } else {
                        // input integer variable
                        do_input_one_var(interpreter, idx, file_handle)?;
                    }
                }
                _ => {
                    do_input_one_var(interpreter, idx, file_handle)?;
                }
            }
        }
        Ok(())
    }

    fn do_input_one_var<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: FileHandle,
    ) -> Result<(), QError> {
        let raw_input: String = if file_handle.is_valid() {
            let file_input = interpreter
                .file_manager()
                .try_get_file_info_input_mut(&file_handle)?;
            file_input.input()?
        } else {
            interpreter.stdin().input()?
        };
        let existing_value = interpreter.context_mut().get_mut(idx).unwrap();
        let temp: &Variant = existing_value;
        let q: TypeQualifier = temp.try_into()?;
        *existing_value = match q {
            TypeQualifier::BangSingle => Variant::from(parse_single_input(raw_input)?),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(parse_int_input(raw_input)?),
            _ => todo!("INPUT type {} not supported yet", q),
        };
        Ok(())
    }

    fn parse_single_input(s: String) -> Result<f32, QError> {
        if s.is_empty() {
            Ok(0.0)
        } else {
            s.parse::<f32>()
                .map_err(|e| format!("Could not parse {} as float: {}", s, e).into())
        }
    }

    fn parse_int_input(s: String) -> Result<i32, QError> {
        if s.is_empty() {
            Ok(0)
        } else {
            s.parse::<i32>()
                .map_err(|e| format!("Could not parse {} as int: {}", s, e).into())
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_has_variable;
        use crate::assert_interpreter_err;
        use crate::interpreter::test_utils::{interpret, interpret_with_raw_input};

        fn assert_input<T>(
            raw_input: &str,
            variable_literal: &str,
            qualified_variable: &str,
            expected_value: T,
        ) where
            Variant: From<T>,
        {
            let input = format!("INPUT {}", variable_literal);
            let interpreter = interpret_with_raw_input(input, raw_input);
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
                assert_input("+3.14", "N", "N!", 3.14_f32);
            }
        }

        mod string_var {
            use super::*;

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

            assert_interpreter_err!(input, QError::InputPastEndOfFile, 5, 13);
            std::fs::remove_file("test_input_string_from_file_eof.txt").unwrap_or_default();
        }
    }
}

mod instr {
    // INSTR([start%,] hay$, needle$)
    // if start% is omitted, INSTR starts at position 1
    // returns the first occurrence of needle$ inside hay$

    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let a: &Variant = interpreter.context().get(0).unwrap();
        let b: &Variant = interpreter.context().get(1).unwrap();
        let result: i32 = match interpreter.context().get(2) {
            Some(c) => do_instr(
                a.try_into().with_err_no_pos()?,
                b.try_into().with_err_no_pos()?,
                c.try_into().with_err_no_pos()?,
            )?,
            None => do_instr(
                1,
                a.try_into().with_err_no_pos()?,
                b.try_into().with_err_no_pos()?,
            )?,
        };
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::InStr.into(), result.into());
        Ok(())
    }

    fn do_instr(start: i32, hay: &String, needle: &String) -> Result<i32, QErrorNode> {
        if start <= 0 {
            Err(QError::IllegalFunctionCall).with_err_no_pos()
        } else if hay.is_empty() {
            Ok(0)
        } else if needle.is_empty() {
            Ok(1)
        } else {
            let mut i: usize = (start - 1) as usize;
            while i + needle.len() <= hay.len() {
                let sub = hay.get(i..(i + needle.len())).unwrap();
                if sub == needle {
                    return Ok((i as i32) + 1);
                }
                i += 1;
            }
            Ok(0)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::assert_prints;
        use crate::common::*;
        use crate::interpreter::interpreter_trait::InterpreterTrait;
        use crate::interpreter::test_utils::interpret_err;

        #[test]
        fn test_instr_happy_flow() {
            assert_prints!(r#"PRINT INSTR("hay", "needle")"#, "0");
            assert_prints!(r#"PRINT INSTR("hay", "hay")"#, "1");
            assert_prints!(r#"PRINT INSTR("hay", "a")"#, "2");
            assert_prints!(r#"PRINT INSTR("hay", "z")"#, "0");
            assert_prints!(r#"PRINT INSTR("hello there", "the")"#, "7");
            assert_prints!(r#"PRINT INSTR(2, "the the", "the")"#, "5");
        }

        #[test]
        fn test_instr_edge_cases() {
            assert_prints!(r#"PRINT INSTR("hay", "")"#, "1");
            assert_prints!(r#"PRINT INSTR("", "needle")"#, "0");
            assert_prints!(r#"PRINT INSTR("", "")"#, "0");
            assert_eq!(
                interpret_err(r#"PRINT INSTR(0, "oops", "zero")"#),
                ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
            );
        }

        #[test]
        fn test_instr_linter() {
            assert_linter_err!(
                r#"PRINT INSTR("oops")"#,
                QError::ArgumentCountMismatch,
                1,
                7
            );
        }
    }
}

mod kill {
    // KILL file-spec$ -> deletes files from disk

    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let file_name: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        std::fs::remove_file(file_name)
            .map_err(|e| e.into())
            .with_err_no_pos()
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::common::*;
        use crate::interpreter::test_utils::*;

        #[test]
        fn test_kill_happy_flow() {
            std::fs::write("KILL1.TXT", "hi").unwrap_or(());
            interpret(r#"KILL "KILL1.TXT""#);
            std::fs::read_to_string("KILL1.TXT").expect_err("File should have been deleted");
        }

        #[test]
        fn test_kill_edge_cases() {
            assert_eq!(
                interpret_err(r#"KILL "KILL2.TXT""#),
                ErrorEnvelope::Pos(QError::FileNotFound, Location::new(1, 1))
            );
        }

        #[test]
        fn test_kill_linter() {
            assert_linter_err!("KILL", QError::ArgumentCountMismatch, 1, 1);
            assert_linter_err!(r#"KILL "a", "b""#, QError::ArgumentCountMismatch, 1, 1);
            assert_linter_err!(r#"KILL 42"#, QError::ArgumentTypeMismatch, 1, 6);
        }
    }
}

mod len {
    // LEN(str_expr$) -> number of characters in string
    // LEN(variable) -> number of bytes required to store a variable
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let v: &Variant = interpreter.context().get(0).unwrap();
        let len: i32 = match v {
            Variant::VSingle(_) => 4,
            Variant::VDouble(_) => 8,
            Variant::VString(v) => v.len().try_into().unwrap(),
            Variant::VInteger(_) => 2,
            Variant::VLong(_) => 4,
            Variant::VUserDefined(user_defined_value) => {
                let user_defined_type = interpreter
                    .user_defined_types()
                    .get(user_defined_value.type_name())
                    .unwrap();
                let sum: u32 =
                    len_of_user_defined_type(user_defined_type, interpreter.user_defined_types());
                sum as i32
            }
            Variant::VArray(_) => todo!(),
        };
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Len.into(), len.into());
        Ok(())
    }

    fn len_of_user_defined_type(
        user_defined_type: &UserDefinedType,
        types: &UserDefinedTypes,
    ) -> u32 {
        let mut sum: u32 = 0;
        for (_, element_type) in user_defined_type.elements() {
            sum += match element_type {
                ElementType::Single => 4,
                ElementType::Double => 8,
                ElementType::Integer => 2,
                ElementType::Long => 4,
                ElementType::FixedLengthString(l) => *l as u32,
                ElementType::UserDefined(type_name) => {
                    len_of_user_defined_type(types.get(type_name).expect("type not found"), types)
                }
            };
        }
        sum
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;
        use crate::interpreter::interpreter_trait::InterpreterTrait;

        #[test]
        fn test_len_string_literal() {
            let program = r#"PRINT LEN("hello")"#;
            assert_prints!(program, "5");
        }

        #[test]
        fn test_len_string_variable() {
            let program = r#"
            A$ = "hello"
            PRINT LEN(A$)
            "#;
            assert_prints!(program, "5");
        }

        #[test]
        fn test_len_float_variable() {
            let program = "
            A = 3.14
            PRINT LEN(A)
            ";
            assert_prints!(program, "4");
        }

        #[test]
        fn test_len_double_variable() {
            let program = "
            A# = 3.14
            PRINT LEN(A#)
            ";
            assert_prints!(program, "8");
        }

        #[test]
        fn test_len_integer_variable() {
            let program = "
            A% = 42
            PRINT LEN(A%)
            ";
            assert_prints!(program, "2");
        }

        #[test]
        fn test_len_long_variable() {
            let program = "
            A& = 42
            PRINT LEN(A&)
            ";
            assert_prints!(program, "4");
        }

        #[test]
        fn test_len_user_defined_type() {
            let program = "
            TYPE Card
                Value AS INTEGER
                Suit AS STRING * 9
            END TYPE
            DIM A AS Card
            PRINT LEN(A)
            ";
            assert_prints!(program, "11");
        }

        #[test]
        fn test_len_user_defined_type_nested_one_level() {
            let program = "
            TYPE PostCode
                Prefix AS STRING * 4
                Suffix AS STRING * 2
            END TYPE
            TYPE Address
                Street AS STRING * 50
                PostCode AS PostCode
            END TYPE
            DIM A AS Address
            PRINT LEN(A)
            ";
            assert_prints!(program, "56");
        }

        #[test]
        fn test_len_user_defined_type_nested_two_levels() {
            let program = "
            TYPE PostCode
                Prefix AS STRING * 4
                Suffix AS STRING * 2
            END TYPE
            TYPE Address
                Street AS STRING * 50
                PostCode AS PostCode
            END TYPE
            TYPE Person
                FullName AS STRING * 100
                Address AS Address
            END TYPE
            DIM A AS Person
            PRINT LEN(A)
            ";
            assert_prints!(program, "156");
        }

        #[test]
        fn test_len_user_defined_type_member() {
            let program = "
            TYPE PostCode
                Prefix AS STRING * 4
                Suffix AS STRING * 2
            END TYPE
            TYPE Address
                Street AS STRING * 50
                PostCode AS PostCode
            END TYPE
            TYPE Person
                FullName AS STRING * 100
                Address AS Address
            END TYPE
            DIM A AS Person
            PRINT LEN(A.Address)
            ";
            assert_prints!(program, "56");
        }
    }
}

mod line_input {
    // LINE INPUT -> see INPUT
    // LINE INPUT [;] ["prompt";] variable$
    // LINE INPUT #file-number%, variable$

    use super::*;
    use crate::interpreter::input::Input;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QError> {
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_file_handle = false;
        for idx in 0..interpreter.context().parameter_count() {
            let v = interpreter.context().get(idx).unwrap();
            match v {
                Variant::VInteger(f) => {
                    if idx == 0 {
                        has_file_handle = *f == 1;
                    } else if idx == 1 {
                        if has_file_handle {
                            file_handle = FileHandle::try_from(*f)?;
                        } else {
                            // input integer variable?
                            panic!("Linter should have caught this");
                        }
                    } else {
                        panic!("Linter should have caught this");
                    }
                }
                Variant::VString(_) => {
                    line_input_one(interpreter, idx, &file_handle)?;
                }
                _ => panic!("Linter should have caught this"),
            }
        }

        Ok(())
    }

    fn line_input_one<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: &FileHandle,
    ) -> Result<(), QError> {
        if file_handle.is_valid() {
            line_input_one_file(interpreter, idx, file_handle)
        } else {
            line_input_one_stdin(interpreter, idx)
        }
    }

    fn line_input_one_file<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
        file_handle: &FileHandle,
    ) -> Result<(), QError> {
        let file_input = interpreter
            .file_manager()
            .try_get_file_info_input_mut(file_handle)?;
        let s = file_input.line_input()?;
        *interpreter.context_mut().get_mut(idx).unwrap() = Variant::VString(s);
        Ok(())
    }

    fn line_input_one_stdin<S: InterpreterTrait>(
        interpreter: &mut S,
        idx: usize,
    ) -> Result<(), QError> {
        let s = interpreter.stdin().input()?;
        *interpreter.context_mut().get_mut(idx).unwrap() = Variant::VString(s);
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use crate::assert_interpreter_err;

        #[test]
        fn test_line_input_string_from_file_eof() {
            std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
            std::fs::write(
                "test_line_input_string_from_file_eof.txt",
                "Hello\r\nWorld\r\n",
            )
            .unwrap();

            let input = r#"
            OPEN "test_line_input_string_from_file_eof.txt" FOR INPUT AS #1
            LINE INPUT #1, A$
            LINE INPUT #1, A$
            LINE INPUT #1, A$ ' should EOF here
            CLOSE
            "#;

            assert_interpreter_err!(input, QError::InputPastEndOfFile, 5, 13);
            std::fs::remove_file("test_line_input_string_from_file_eof.txt").unwrap_or_default();
        }
    }
}

mod mid {
    // MID$ function returns part of a string
    // MID$ statement replaces part of a string (TODO support this)
    // MID$(str_expr$, start%[, length%])
    // MID$(str_var$, start%[, length%]) = str_expr$
    // if the length is omitted, returns or replaces all remaining characters
    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let s: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let start: i32 = interpreter
            .context()
            .get(1)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let length: Option<i32> = match interpreter.context().get(2) {
            Some(v) => Some(v.try_into().with_err_no_pos()?),
            None => None,
        };
        let result: String = do_mid(s, start, length)?;
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Mid.into(), result.into());
        Ok(())
    }

    fn do_mid(s: &String, start: i32, opt_length: Option<i32>) -> Result<String, QErrorNode> {
        if start <= 0 {
            Err(QError::IllegalFunctionCall).with_err_no_pos()
        } else {
            let start_idx: usize = (start - 1) as usize;
            match opt_length {
                Some(length) => {
                    if length < 0 {
                        Err(QError::IllegalFunctionCall).with_err_no_pos()
                    } else {
                        let end: usize = if start_idx + (length as usize) > s.len() {
                            s.len()
                        } else {
                            start_idx + (length as usize)
                        };
                        Ok(s.get(start_idx..end).unwrap_or_default().to_string())
                    }
                }
                None => Ok(s.get(start_idx..).unwrap_or_default().to_string()),
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::assert_prints;
        use crate::common::*;
        use crate::interpreter::interpreter_trait::InterpreterTrait;
        use crate::interpreter::test_utils::interpret_err;
        #[test]
        fn test_mid_happy_flow() {
            assert_prints!(r#"PRINT MID$("hay", 1)"#, "hay");
            assert_prints!(r#"PRINT MID$("hay", 2)"#, "ay");
            assert_prints!(r#"PRINT MID$("hay", 1, 1)"#, "h");
            assert_prints!(r#"PRINT MID$("hay", 2, 2)"#, "ay");
            assert_prints!(r#"PRINT MID$("hay", 2, 20)"#, "ay");
        }

        #[test]
        fn test_mid_edge_cases() {
            assert_prints!(r#"PRINT MID$("", 1)"#, "");
            assert_prints!(r#"PRINT MID$("hay", 4)"#, "");
            assert_prints!(r#"PRINT MID$("hay", 1, 0)"#, "");
            assert_eq!(
                interpret_err(r#"PRINT MID$("hay", 0)"#),
                ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
            );
            assert_eq!(
                interpret_err(r#"PRINT MID$("hay", 1, -1)"#),
                ErrorEnvelope::Pos(QError::IllegalFunctionCall, Location::new(1, 7))
            );
        }

        #[test]
        fn test_mid_linter() {
            assert_linter_err!(r#"PRINT MID$("oops")"#, QError::ArgumentCountMismatch, 1, 7);
        }
    }
}

mod name {
    // NAME old$ AS new$
    // Renames a file or directory.

    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let old_file_name: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let new_file_name: &String = interpreter
            .context()
            .get(1)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        std::fs::rename(old_file_name, new_file_name)
            .map_err(|e| e.into())
            .with_err_no_pos()
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::common::*;
        use crate::interpreter::test_utils::*;

        #[test]
        fn test_can_rename_file() {
            // arrange
            std::fs::write("TEST4.OLD", "hi").unwrap_or(());
            let input = r#"
            NAME "TEST4.OLD" AS "TEST4.NEW"
            "#;
            // act
            interpret(input);
            // assert
            let contents = std::fs::read_to_string("TEST4.NEW").unwrap_or("".to_string());
            std::fs::remove_file("TEST4.OLD").unwrap_or(());
            std::fs::remove_file("TEST4.NEW").unwrap_or(());
            assert_eq!(contents, "hi");
        }

        #[test]
        fn test_can_rename_file_parenthesis() {
            // arrange
            std::fs::write("TEST5.OLD", "hi").unwrap_or(());
            let input = r#"
            NAME("TEST5.OLD")AS("TEST5.NEW")
            "#;
            // act
            interpret(input);
            // assert
            let contents = std::fs::read_to_string("TEST5.NEW").unwrap_or("".to_string());
            std::fs::remove_file("TEST5.OLD").unwrap_or(());
            std::fs::remove_file("TEST5.NEW").unwrap_or(());
            assert_eq!(contents, "hi");
        }

        #[test]
        fn test_can_rename_directory() {
            // arrange
            let old_dir_name = "TEST.DIR";
            let new_dir_name = "NEW.DIR";
            std::fs::remove_dir(old_dir_name).unwrap_or(());
            std::fs::remove_dir(new_dir_name).unwrap_or(());
            std::fs::create_dir(old_dir_name).expect("Should have created directory");

            // act
            interpret(format!(r#"NAME "{}" AS "{}""#, old_dir_name, new_dir_name));

            // assert
            std::fs::metadata(old_dir_name).expect_err("should fail");
            let attr = std::fs::metadata(new_dir_name).expect("should succeed");
            assert!(attr.is_dir());
            std::fs::remove_dir(old_dir_name).unwrap_or(());
            std::fs::remove_dir(new_dir_name).unwrap_or(());
        }

        #[test]
        fn test_name_linter_err() {
            assert_linter_err!(r#"NAME 1 AS "boo""#, QError::ArgumentTypeMismatch, 1, 6);
            assert_linter_err!(r#"NAME "boo" AS 2"#, QError::ArgumentTypeMismatch, 1, 15);
        }
    }
}

mod open {
    // OPEN file$ [FOR mode] [ACCESS access] [lock] AS [#]file-number% [LEN=rec-len%]
    //
    // mode: APPEND, BINARY, INPUT, OUTPUT, RANDOM
    // access: READ, WRITE, READ WRITE
    // lock: SHARED, LOCK READ, LOCK WRITE, LOCK READ WRITE
    // file-number a number in the range 1 through 255
    // rec-len%: For random access files, the record length (default is 128 bytes)
    //           For sequential files, the number of characters buffered (default is 512 bytes)

    use super::*;
    use std::convert::TryFrom;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let file_name: String = interpreter.context().get(0).unwrap().to_string();
        let file_mode: FileMode = i32::try_from(interpreter.context().get(1).unwrap())
            .with_err_no_pos()?
            .into();
        let file_access: FileAccess = i32::try_from(interpreter.context().get(2).unwrap())
            .with_err_no_pos()?
            .into();
        let file_handle: FileHandle = interpreter
            .context()
            .get(3)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        interpreter
            .file_manager()
            .open(file_handle, &file_name, file_mode, file_access)
            .with_err_no_pos()
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_interpreter_err;
        use crate::assert_prints;
        use crate::common::QError;
        use crate::interpreter::interpreter_trait::InterpreterTrait;
        use crate::interpreter::test_utils::*;

        #[test]
        fn test_can_create_file() {
            std::fs::remove_file("TEST1.TXT").unwrap_or(());
            let input = r#"
            OPEN "TEST1.TXT" FOR APPEND AS #1
            PRINT #1, "Hello, world"
            CLOSE #1
            "#;
            interpret(input);
            let contents = std::fs::read_to_string("TEST1.TXT").unwrap_or("".to_string());
            std::fs::remove_file("TEST1.TXT").unwrap_or(());
            assert_eq!("Hello, world\r\n", contents);
        }

        #[test]
        fn test_can_read_file() {
            let input = r#"
            OPEN "TEST2A.TXT" FOR APPEND AS #1
            PRINT #1, "Hello, world"
            CLOSE #1
            OPEN "TEST2A.TXT" FOR INPUT AS #1
            LINE INPUT #1, T$
            CLOSE #1
            OPEN "TEST2B.TXT" FOR APPEND AS #1
            PRINT #1, T$
            CLOSE #1
            "#;
            interpret(input);
            let contents = std::fs::read_to_string("TEST2B.TXT").unwrap_or("".to_string());
            std::fs::remove_file("TEST2A.TXT").unwrap_or(());
            std::fs::remove_file("TEST2B.TXT").unwrap_or(());
            assert_eq!("Hello, world\r\n", contents);
        }

        #[test]
        fn test_can_read_file_until_eof() {
            std::fs::remove_file("TEST3.TXT").unwrap_or(());
            let input = r#"
            OPEN "TEST3.TXT" FOR APPEND AS #1
            PRINT #1, "Hello, world"
            PRINT #1, "Hello, again"
            CLOSE #1
            OPEN "TEST3.TXT" FOR INPUT AS #1
            WHILE NOT EOF(1)
            LINE INPUT #1, T$
            PRINT T$
            WEND
            CLOSE #1
            "#;
            assert_prints!(input, "Hello, world", "Hello, again");
            std::fs::remove_file("TEST3.TXT").unwrap_or(());
        }

        #[test]
        fn test_can_write_file_append_mode() {
            std::fs::remove_file("test_can_write_file_append_mode.TXT").unwrap_or(());
            let input = r#"
            OPEN "test_can_write_file_append_mode.TXT" FOR APPEND AS #1
            PRINT #1, "Hello, world"
            PRINT #1, "Hello, again"
            CLOSE #1
            "#;
            interpret(input);
            let read_result = std::fs::read_to_string("test_can_write_file_append_mode.TXT");
            std::fs::remove_file("test_can_write_file_append_mode.TXT").unwrap_or(());
            assert_eq!(read_result.unwrap(), "Hello, world\r\nHello, again\r\n");
        }

        #[test]
        fn test_open_bad_file_number_runtime_error() {
            let input = r#"
            A = 256
            OPEN "TEST.TXT" FOR INPUT AS A
            CLOSE A
            "#;
            assert_interpreter_err!(input, QError::BadFileNameOrNumber, 3, 13);
        }

        #[test]
        fn test_open_twice_without_closing_error() {
            let input = r#"
            OPEN "a.txt" FOR OUTPUT AS #1
            OPEN "a.txt" FOR OUTPUT AS #1
            "#;
            assert_interpreter_err!(input, QError::FileAlreadyOpen, 3, 13);
            std::fs::remove_file("a.txt").unwrap_or(());
        }
    }
}

mod str_fn {
    // STR$(numeric-expression) returns a string representation of a number

    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let v: &Variant = interpreter.context().get(0).unwrap();
        let result = match v {
            Variant::VSingle(f) => format!("{}", f),
            Variant::VDouble(f) => format!("{}", f),
            Variant::VInteger(f) => format!("{}", f),
            Variant::VLong(f) => format!("{}", f),
            _ => panic!("unexpected arg to STR$"),
        };
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Str.into(), result.into());
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;
        use crate::interpreter::interpreter_trait::InterpreterTrait;

        #[test]
        fn test_str_float() {
            let program = r#"PRINT STR$(3.14)"#;
            assert_prints!(program, "3.14");
        }
    }
}

mod system {
    use super::*;

    pub fn run<S: InterpreterTrait>(_interpreter: &mut S) -> Result<(), QErrorNode> {
        panic!("Should have been handled at the IG level")
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::common::QError;

        #[test]
        fn test_sub_call_system_no_args_allowed() {
            assert_linter_err!("SYSTEM 42", QError::ArgumentCountMismatch, 1, 1);
        }
    }
}

mod val {
    // VAL(str-expr$) converts a string representation of a number to a number.

    use super::*;

    pub fn run<S: InterpreterTrait>(interpreter: &mut S) -> Result<(), QErrorNode> {
        let v: &String = interpreter
            .context()
            .get(0)
            .unwrap()
            .try_into()
            .with_err_no_pos()?;
        let result: Variant = val(v).with_err_no_pos()?;
        interpreter
            .context_mut()
            .set_variable(BuiltInFunction::Val.into(), result.into());
        Ok(())
    }

    fn val(s: &String) -> Result<Variant, QError> {
        let mut is_positive = true;
        let mut value: f64 = 0.0;
        let mut frac_power: i32 = 0;

        const STATE_INITIAL: u8 = 0;
        const STATE_SIGN: u8 = 1;
        const STATE_INT: u8 = 2;
        const STATE_DOT: u8 = 3;
        const STATE_FRAC: u8 = 4;
        let mut state: u8 = STATE_INITIAL;

        for c in s.chars() {
            if c >= '0' && c <= '9' {
                if state == STATE_INITIAL || state == STATE_SIGN {
                    state = STATE_INT;
                } else if state == STATE_DOT {
                    state = STATE_FRAC;
                }
                if state == STATE_INT {
                    value = value * 10.0 + ((c as u8) - ('0' as u8)) as f64;
                } else {
                    if frac_power <= MAX_INTEGER {
                        frac_power += 1;
                    } else {
                        return Err(QError::Overflow);
                    }
                    value = (value * 10.0_f64.powi(frac_power) + ((c as u8) - ('0' as u8)) as f64)
                        / 10.0_f64.powi(frac_power);
                }
            } else if c == ' ' {
                // ignore spaces apparently
            } else if c == '.' {
                if state == STATE_DOT || state == STATE_FRAC {
                    break;
                } else {
                    state = STATE_DOT;
                }
            } else if c == '-' {
                if state == STATE_INITIAL {
                    state = STATE_SIGN;
                    is_positive = false;
                } else {
                    break;
                }
            } else if c == '+' {
                if state == STATE_INITIAL {
                    state = STATE_SIGN;
                } else {
                    break;
                }
            } else {
                // bail out
                break;
            }
        }

        if state == STATE_INITIAL || state == STATE_SIGN {
            Ok(Variant::VInteger(0))
        } else if state == STATE_INT || state == STATE_DOT {
            if is_positive && value <= MAX_INTEGER as f64 {
                Ok(Variant::VInteger(value as i32))
            } else if !is_positive && value <= (1 + MAX_INTEGER) as f64 {
                Ok(Variant::VInteger(-value as i32))
            } else if is_positive && value <= MAX_LONG as f64 {
                Ok(Variant::VLong(value as i64))
            } else if !is_positive && value <= (1 + MAX_LONG) as f64 {
                Ok(Variant::VLong(-value as i64))
            } else {
                let x = Variant::VDouble(value);
                if is_positive {
                    Ok(x)
                } else {
                    x.negate()
                }
            }
        } else {
            let x = Variant::VDouble(value);
            if is_positive {
                Ok(x)
            } else {
                x.negate()
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;
        use crate::interpreter::interpreter_trait::InterpreterTrait;

        #[test]
        fn test_val_float() {
            let program = r#"PRINT VAL("3.14")"#;
            assert_prints!(program, "3.14");
        }

        #[test]
        fn test_val_integer() {
            let program = r#"PRINT VAL("42")"#;
            assert_prints!(program, "42");
        }

        #[test]
        fn test_val_invalid_string_gives_zero() {
            let program = r#"PRINT VAL("oops")"#;
            assert_prints!(program, "0");
        }

        #[test]
        fn test_val_partial_parse() {
            let program = r#"PRINT VAL("3.14oops")"#;
            assert_prints!(program, "3.14");
        }

        #[test]
        fn test_val_partial_parse_ignores_spaces() {
            let program = r#"PRINT VAL("  -    4   . 2   ")"#;
            assert_prints!(program, "-4.2");
        }

        #[test]
        fn test_val_no_overflow() {
            let program = r#"PRINT VAL("1234567890123456789012345678901234567890")"#;
            assert_prints!(program, "1234567890123456800000000000000000000000");
        }
    }
}
