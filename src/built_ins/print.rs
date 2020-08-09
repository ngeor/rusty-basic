// PRINT [#file-number%,] [expression-list] [{; | ,}]
// ; -> output immediately after the last value
// , -> print at the start of the next print zone (print zones are 14 characters wide)

use super::{BuiltInLint, BuiltInRun};
use crate::common::*;
use crate::interpreter::{Interpreter, InterpreterErrorNode, Stdlib};
use crate::linter::{ExpressionNode, LinterErrorNode};
use crate::variant::Variant;

pub struct Print {}

impl BuiltInLint for Print {
    fn lint(&self, _args: &Vec<ExpressionNode>) -> Result<(), LinterErrorNode> {
        Ok(())
    }
}

impl BuiltInRun for Print {
    fn run<S: Stdlib>(&self, interpreter: &mut Interpreter<S>) -> Result<(), InterpreterErrorNode> {
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
