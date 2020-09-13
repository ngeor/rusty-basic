use crate::built_ins::{BuiltInFunction, BuiltInSub};
use crate::common::{
    FileAccess, FileHandle, FileMode, QError, QErrorNode,
    ToErrorEnvelopeNoPos,
};
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{Interpreter, Stdlib};
use crate::linter::{
    ResolvedDeclaredName, ResolvedElement, ResolvedElementType, ResolvedTypeDefinition,
    ResolvedUserDefinedType, ResolvedUserDefinedTypes
};
use crate::parser::TypeQualifier;
use crate::variant::{Variant, MAX_INTEGER, MAX_LONG};
use std::convert::TryInto;

pub fn run_function<S: Stdlib>(
    f: &BuiltInFunction,
    interpreter: &mut Interpreter<S>,
) -> Result<(), QErrorNode> {
    match f {
        BuiltInFunction::Chr => chr::run(interpreter),
        BuiltInFunction::Environ => environ_fn::run(interpreter),
        BuiltInFunction::Eof => eof::run(interpreter),
        BuiltInFunction::InStr => instr::run(interpreter),
        BuiltInFunction::Len => len::run(interpreter),
        BuiltInFunction::Mid => mid::run(interpreter),
        BuiltInFunction::Str => str_fn::run(interpreter),
        BuiltInFunction::Val => val::run(interpreter),
    }
}

pub fn run_sub<S: Stdlib>(
    s: &BuiltInSub,
    interpreter: &mut Interpreter<S>,
) -> Result<(), QErrorNode> {
    match s {
        BuiltInSub::Close => close::run(interpreter),
        BuiltInSub::Environ => environ_sub::run(interpreter),
        BuiltInSub::Input => input::run(interpreter),
        BuiltInSub::Kill => kill::run(interpreter),
        BuiltInSub::LineInput => line_input::run(interpreter),
        BuiltInSub::Name => name::run(interpreter),
        BuiltInSub::Open => open::run(interpreter),
        BuiltInSub::Print => print::run(interpreter),
        BuiltInSub::System => system::run(interpreter),
    }
}

mod chr {
    // CHR$(ascii-code%) returns the text representation of the given ascii code
    use super::*;
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let i: i32 = interpreter.pop_integer();
        let mut s: String = String::new();
        s.push((i as u8) as char);
        interpreter.function_result = s.into();
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_linter_err;
        use crate::assert_prints;
        use crate::common::QError;

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
    // TODO : close without arguments closes all files
    use super::*;
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_handle = interpreter.pop_file_handle().with_err_no_pos()?;
        interpreter.file_manager.close(file_handle);
        Ok(())
    }
}

mod environ_fn {
    // ENVIRON$ (env-variable$) -> returns the variable
    // ENVIRON$ (n%) -> returns the nth variable (TODO support this)
    use super::*;
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        match v {
            Variant::VString(env_var_name) => {
                let result = interpreter.stdlib.get_env_var(&env_var_name);
                interpreter.function_result = Variant::VString(result);
                Ok(())
            }
            _ => panic!("Type mismatch at ENVIRON$",),
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_has_variable;
        use crate::assert_linter_err;
        use crate::common::QError;
        use crate::interpreter::test_utils::*;
        use crate::interpreter::Stdlib;

        #[test]
        fn test_function_call_environ() {
            let program = r#"
            X$ = ENVIRON$("abc")
            Y$ = ENVIRON$("def")
            "#;
            let mut stdlib = MockStdlib::new();
            stdlib.set_env_var("abc".to_string(), "foo".to_string());
            let interpreter = interpret_with_stdlib(program, stdlib);
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
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        match interpreter.pop_unnamed_val().unwrap() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    Err(QError::from("Invalid expression. Must be name=value.")).with_err_no_pos()
                } else {
                    interpreter
                        .stdlib
                        .set_env_var(parts[0].to_string(), parts[1].to_string());
                    Ok(())
                }
            }
            _ => panic!("Type mismatch"),
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::interpreter::stdlib::Stdlib;
        use crate::interpreter::test_utils::interpret;

        #[test]
        fn test_sub_call_environ() {
            let program = r#"
            ENVIRON "FOO=BAR"
            "#;
            let interpreter = interpret(program);
            assert_eq!(interpreter.stdlib.get_env_var(&"FOO".to_string()), "BAR");
        }
    }
}

mod eof {
    // EOF(file-number%) -> checks if the end of file has been reached
    use super::*;
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_handle: FileHandle = interpreter.pop_file_handle().with_err_no_pos()?;
        let is_eof: bool = interpreter
            .file_manager
            .eof(file_handle)
            .map_err(|e| e.into())
            .with_err_no_pos()?;
        interpreter.function_result = is_eof.into();
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
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        loop {
            match &interpreter.pop_unnamed_arg() {
                Some(a) => match a {
                    Argument::ByRef(n) => {
                        do_input_one_var(interpreter, a, n)?;
                    }
                    _ => {
                        panic!("Expected: variable (linter should have caught this)");
                    }
                },
                None => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn do_input_one_var<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        a: &Argument,
        n: &ResolvedDeclaredName,
    ) -> Result<(), QErrorNode> {
        let raw_input: String = interpreter
            .stdlib
            .input()
            .map_err(|e| e.into())
            .with_err_no_pos()?;
        let variable_value = match n.type_definition() {
            ResolvedTypeDefinition::BuiltIn(TypeQualifier::BangSingle) => {
                Variant::from(parse_single_input(raw_input).with_err_no_pos()?)
            }
            ResolvedTypeDefinition::BuiltIn(TypeQualifier::DollarString) => {
                Variant::from(raw_input)
            }
            ResolvedTypeDefinition::BuiltIn(TypeQualifier::PercentInteger) => {
                Variant::from(parse_int_input(raw_input).with_err_no_pos()?)
            }
            _ => unimplemented!(),
        };
        // TODO casting
        interpreter
            .context_mut()
            .demand_sub()
            .set_value_to_popped_arg(a, variable_value);
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
        use crate::interpreter::test_utils::{interpret_with_stdlib, MockStdlib};

        fn assert_input<T>(
            raw_input: &str,
            variable_literal: &str,
            qualified_variable: &str,
            expected_value: T,
        ) where
            Variant: From<T>,
        {
            let mut stdlib = MockStdlib::new();
            stdlib.add_next_input(raw_input);
            let input = format!("INPUT {}", variable_literal);
            let interpreter = interpret_with_stdlib(input, stdlib);
            assert_has_variable!(interpreter, qualified_variable, expected_value);
        }

        mod unqualified_var {
            use super::*;

            #[test]
            fn test_input_empty() {
                assert_input("", "N", "N!", 0.0_f32);
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
            fn test_input_hello() {
                assert_input("hello", "A$", "A$", "hello");
            }

            #[test]
            fn test_input_does_not_trim_new_line() {
                assert_input("hello\r\n", "A$", "A$", "hello\r\n");
            }
        }

        mod int_var {
            use super::*;

            #[test]
            fn test_input_42() {
                assert_input("42", "A%", "A%", 42);
            }
        }
    }
}

mod instr {
    // INSTR([start%,] hay$, needle$)
    // if start% is omitted, INSTR starts at position 1
    // returns the first occurrence of needle$ inside hay$

    use super::*;
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let a: Variant = interpreter.pop_unnamed_val().unwrap();
        let b: Variant = interpreter.pop_unnamed_val().unwrap();
        let result: i32 = match interpreter.pop_unnamed_val() {
            Some(c) => do_instr(a.demand_integer(), b.demand_string(), c.demand_string())?,
            None => do_instr(1, a.demand_string(), b.demand_string())?,
        };
        interpreter.function_result = result.into();
        Ok(())
    }

    fn do_instr(start: i32, hay: String, needle: String) -> Result<i32, QErrorNode> {
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
    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_name = interpreter.pop_string();
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

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VSingle(_) => Variant::VInteger(4),
            Variant::VDouble(_) => Variant::VInteger(8),
            Variant::VString(v) => Variant::VInteger(v.len().try_into().unwrap()),
            Variant::VInteger(_) => Variant::VInteger(2),
            Variant::VLong(_) => Variant::VInteger(4),
            Variant::VFileHandle(_) => {
                return Err("File handle not supported".into()).with_err_no_pos();
            }
            Variant::VUserDefined(user_defined_value) => {
                let user_defined_type = interpreter
                    .user_defined_types
                    .as_ref()
                    .get(user_defined_value.type_name())
                    .unwrap();
                let sum: u32 = len_of_user_defined_type(
                    user_defined_type,
                    interpreter.user_defined_types.as_ref(),
                );
                Variant::VInteger(sum as i32)
            }
        };
        Ok(())
    }

    fn len_of_user_defined_type(
        user_defined_type: &ResolvedUserDefinedType,
        types: &ResolvedUserDefinedTypes,
    ) -> u32 {
        let mut sum: u32 = 0;
        for ResolvedElement { element_type, .. } in user_defined_type.elements.iter() {
            sum += match element_type {
                ResolvedElementType::Single => 4,
                ResolvedElementType::Double => 8,
                ResolvedElementType::Integer => 2,
                ResolvedElementType::Long => 4,
                ResolvedElementType::String(l) => *l,
                ResolvedElementType::UserDefined(type_name) => {
                    len_of_user_defined_type(types.get(type_name).expect("type not found"), types)
                }
            };
        }
        sum
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;

        #[test]
        fn test_len_string() {
            let program = r#"PRINT LEN("hello")"#;
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

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let mut is_first = true;
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_more = true;
        while has_more {
            match interpreter.pop_unnamed_arg() {
                Some(a) => {
                    let arg_ref = &a;
                    match arg_ref {
                        Argument::ByVal(v) => {
                            if is_first {
                                match v {
                                    Variant::VFileHandle(f) => {
                                        file_handle = *f;
                                    }
                                    _ => {
                                        panic!("LINE INPUT linter should have caught this");
                                    }
                                }
                            } else {
                                panic!("LINE INPUT linter should have caught this");
                            }
                        }
                        Argument::ByRef(n) => {
                            line_input_one(interpreter, arg_ref, n, file_handle)?;
                        }
                    }
                    is_first = false;
                }
                None => {
                    has_more = false;
                }
            }
        }
        Ok(())
    }

    fn line_input_one<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        arg: &Argument,
        n: &ResolvedDeclaredName,
        file_handle: FileHandle,
    ) -> Result<(), QErrorNode> {
        if file_handle.is_valid() {
            line_input_one_file(interpreter, arg, n, file_handle)
        } else {
            line_input_one_stdin(interpreter, arg, n)
        }
    }

    fn line_input_one_file<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        arg: &Argument,
        n: &ResolvedDeclaredName,
        file_handle: FileHandle,
    ) -> Result<(), QErrorNode> {
        let s = interpreter
            .file_manager
            .read_line(file_handle)
            .map_err(|e| e.into())
            .with_err_no_pos()?;
        match n.type_definition() {
            ResolvedTypeDefinition::BuiltIn(TypeQualifier::DollarString) => {
                // TODO casting
                interpreter
                    .context_mut()
                    .demand_sub()
                    .set_value_to_popped_arg(arg, Variant::VString(s));
                Ok(())
            }
            _ => unimplemented!(),
        }
    }

    fn line_input_one_stdin<S: Stdlib>(
        interpreter: &mut Interpreter<S>,
        arg: &Argument,
        _n: &ResolvedDeclaredName,
    ) -> Result<(), QErrorNode> {
        let s = interpreter
            .stdlib
            .input()
            .map_err(|e| e.into())
            .with_err_no_pos()?;
        // TODO casting
        interpreter
            .context_mut()
            .demand_sub()
            .set_value_to_popped_arg(arg, Variant::VString(s));
        Ok(())
    }
}

mod mid {
    // MID$ function returns part of a string
    // MID$ statement replaces part of a string (TODO support this)
    // MID$(str_expr$, start%[, length%])
    // MID$(str_var$, start%[, length%]) = str_expr$
    // if the length is omitted, returns or replaces all remaining characters
    use super::*;

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let s: String = interpreter.pop_string();
        let start: i32 = interpreter.pop_integer();
        let length: Option<i32> = interpreter.pop_unnamed_val().map(|v| v.demand_integer());
        let result: String = do_mid(s, start, length)?;
        interpreter.function_result = result.into();
        Ok(())
    }

    fn do_mid(s: String, start: i32, opt_length: Option<i32>) -> Result<String, QErrorNode> {
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
    // TODO support directory

    use super::*;

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let old_file_name = interpreter.pop_string();
        let new_file_name = interpreter.pop_string();
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
    // file-number a number in the range 1 through 255 (TODO enforce this)
    // rec-len%: For random access files, the record length (default is 128 bytes)
    //           For sequential files, the number of characters buffered (default is 512 bytes)

    use super::*;

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let file_name = interpreter.pop_string();
        let file_mode: FileMode = interpreter.pop_integer().into();
        let file_access: FileAccess = interpreter.pop_integer().into();
        let file_handle = interpreter.pop_file_handle().with_err_no_pos()?;
        interpreter
            .file_manager
            .open(file_handle, file_name.as_ref(), file_mode, file_access)
            .map_err(|e| e.into())
            .with_err_no_pos()
    }

    #[cfg(test)]
    mod tests {
        use crate::interpreter::test_utils::*;
        use crate::interpreter::DefaultStdlib;
        use crate::interpreter::Interpreter;

        #[test]
        fn test_can_create_file() {
            let input = r#"
            OPEN "TEST1.TXT" FOR APPEND AS #1
            PRINT #1, "Hello, world"
            CLOSE #1
            "#;
            let (instructions, user_defined_types) = generate_instructions(input);
            let mut interpreter = Interpreter::new(DefaultStdlib {}, user_defined_types);
            interpreter.interpret(instructions).unwrap_or_default();
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
            let (instructions, user_defined_types) = generate_instructions(input);
            let mut interpreter = Interpreter::new(DefaultStdlib {}, user_defined_types);
            interpreter.interpret(instructions).unwrap_or_default();
            let contents = std::fs::read_to_string("TEST2B.TXT").unwrap_or("".to_string());
            std::fs::remove_file("TEST2A.TXT").unwrap_or(());
            std::fs::remove_file("TEST2B.TXT").unwrap_or(());
            assert_eq!("Hello, world\r\n", contents);
        }

        #[test]
        fn test_can_read_file_until_eof() {
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
            let (instructions, user_defined_types) = generate_instructions(input);
            let stdlib = MockStdlib::new();
            let mut interpreter = Interpreter::new(stdlib, user_defined_types);
            interpreter.interpret(instructions).unwrap_or_default();
            std::fs::remove_file("TEST3.TXT").unwrap_or(());
            assert_eq!(
                interpreter.stdlib.output,
                vec!["Hello, world", "Hello, again"]
            );
        }
    }
}

mod print {
    // PRINT [#file-number%,] [expression-list] [{; | ,}]
    // ; -> output immediately after the last value
    // , -> print at the start of the next print zone (print zones are 14 characters wide)

    use super::*;

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let mut print_args: Vec<String> = vec![];
        let mut is_first = true;
        let mut file_handle: FileHandle = FileHandle::default();
        loop {
            match interpreter.pop_unnamed_val() {
                Some(v) => match v {
                    Variant::VFileHandle(fh) => {
                        if is_first {
                            file_handle = fh;
                            is_first = false;
                        } else {
                            panic!("file handle must be first")
                        }
                    }
                    _ => print_args.push(v.to_string()),
                },
                None => {
                    break;
                }
            }
        }
        if file_handle.is_valid() {
            interpreter
                .file_manager
                .print(file_handle, print_args)
                .map_err(|e| e.into())
                .with_err_no_pos()?;
        } else {
            interpreter.stdlib.print(print_args);
        }
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;

        #[test]
        fn test_print_no_args() {
            assert_prints!("PRINT", "");
        }

        #[test]
        fn test_interpret_print_hello_world_one_arg() {
            let input = "PRINT \"Hello, world!\"";
            assert_prints!(input, "Hello, world!");
        }

        #[test]
        fn test_interpret_print_hello_world_two_args() {
            let input = r#"PRINT "Hello", "world!""#;
            assert_prints!(input, "Hello world!");
        }

        #[test]
        fn test_interpret_print_hello_world_two_args_one_is_function() {
            let input = r#"
        PRINT "Hello", Test(1)
        FUNCTION Test(N)
            Test = N + 1
        END FUNCTION
        "#;
            assert_prints!(input, "Hello 2");
        }
    }
}

mod str_fn {
    // STR$(numeric-expression) returns a string representation of a number
    // TODO support hexadecimal literals &H10

    use super::*;

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VSingle(f) => Variant::VString(format!("{}", f)),
            Variant::VDouble(f) => Variant::VString(format!("{}", f)),
            Variant::VInteger(f) => Variant::VString(format!("{}", f)),
            Variant::VLong(f) => Variant::VString(format!("{}", f)),
            _ => panic!("unexpected arg to STR$"),
        };
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use crate::assert_prints;

        #[test]
        fn test_str_float() {
            let program = r#"PRINT STR$(3.14)"#;
            assert_prints!(program, "3.14");
        }
    }
}

mod system {
    use super::*;

    pub fn run<S: Stdlib>(_interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
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

    pub fn run<S: Stdlib>(interpreter: &mut Interpreter<S>) -> Result<(), QErrorNode> {
        let v = interpreter.pop_unnamed_val().unwrap();
        interpreter.function_result = match v {
            Variant::VString(s) => val(s).with_err_no_pos()?,
            _ => panic!("unexpected arg to VAL"),
        };
        Ok(())
    }

    fn val(s: String) -> Result<Variant, QError> {
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
