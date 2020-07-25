use crate::built_ins;
use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{err_no_pos, Interpreter, InterpreterError, Result, Stdlib};
use crate::linter::{BuiltInSub, HasQualifier, QualifiedName, TypeQualifier};
use crate::variant::Variant;

mod line_input;

impl<S: Stdlib> Interpreter<S> {
    pub fn run_built_in_sub(&mut self, name: &BuiltInSub, pos: Location) -> Result<()> {
        match name {
            BuiltInSub::Print => {
                let mut print_args: Vec<String> = vec![];
                let mut is_first = true;
                let mut file_handle: FileHandle = FileHandle::default();
                loop {
                    match self.pop_unnamed_val() {
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
                    self.file_manager
                        .print(file_handle, print_args)
                        .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))?;
                } else {
                    self.stdlib.print(print_args);
                }
                Ok(())
            }
            BuiltInSub::Environ => self.do_environ_sub().map_err(|e| e.at(pos)),
            BuiltInSub::Input => self.do_input().map_err(|e| e.at(pos)),
            BuiltInSub::System => panic!("Should have been handled at the IG level"),
            BuiltInSub::Close => {
                let file_handle = self.pop_file_handle();
                self.file_manager.close(file_handle);
                Ok(())
            }
            BuiltInSub::Open => {
                let file_name = self.pop_string();
                let file_mode: FileMode = self.pop_integer().into();
                let file_access: FileAccess = self.pop_integer().into();
                let file_handle = self.pop_file_handle();
                self.file_manager
                    .open(file_handle, file_name.as_ref(), file_mode, file_access)
                    .map_err(|e| {
                        InterpreterError::new_with_pos(
                            format!("Could not open {}: {}", file_name, e),
                            pos,
                        )
                    })
            }
            BuiltInSub::LineInput => self.line_input().map_err(|e| e.at(pos)),
            BuiltInSub::Name => built_ins::name::run(self, pos),
        }
    }

    fn do_environ_sub(&mut self) -> Result<()> {
        match self.pop_unnamed_val().unwrap() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    err_no_pos("Invalid expression. Must be name=value.")
                } else {
                    self.stdlib
                        .set_env_var(parts[0].to_string(), parts[1].to_string());
                    Ok(())
                }
            }
            _ => panic!("Type mismatch"),
        }
    }

    fn do_input(&mut self) -> Result<()> {
        loop {
            match &self.pop_unnamed_arg() {
                Some(a) => match a {
                    Argument::ByRef(n) => {
                        self.do_input_one_var(a, n)?;
                    }
                    _ => {
                        panic!("Expected variable (linter should have caught this)");
                    }
                },
                None => {
                    break;
                }
            }
        }
        Ok(())
    }

    fn do_input_one_var(&mut self, a: &Argument, n: &QualifiedName) -> Result<()> {
        let raw_input: String = self
            .stdlib
            .input()
            .map_err(|e| InterpreterError::new_no_pos(e.to_string()))?;
        let q: TypeQualifier = n.qualifier();
        let variable_value = match q {
            TypeQualifier::BangSingle => Variant::from(
                parse_single_input(raw_input).map_err(|e| InterpreterError::new_no_pos(e))?,
            ),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(
                parse_int_input(raw_input).map_err(|e| InterpreterError::new_no_pos(e))?,
            ),
            _ => unimplemented!(),
        };
        self.context_mut()
            .demand_sub()
            .set_value_to_popped_arg(a, variable_value)
            .map_err(|e| InterpreterError::new_no_pos(e))
    }
}

fn parse_single_input(s: String) -> std::result::Result<f32, String> {
    if s.is_empty() {
        Ok(0.0)
    } else {
        s.parse::<f32>()
            .map_err(|e| format!("Could not parse {} as float: {}", s, e))
    }
}

fn parse_int_input(s: String) -> std::result::Result<i32, String> {
    if s.is_empty() {
        Ok(0)
    } else {
        s.parse::<i32>()
            .map_err(|e| format!("Could not parse {} as int: {}", s, e))
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    #[test]
    fn test_sub_call_system_no_args_allowed() {
        assert_linter_err!("SYSTEM 42", LinterError::ArgumentCountMismatch, 1, 1);
    }
}
