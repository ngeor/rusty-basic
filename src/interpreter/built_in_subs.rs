use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{err, Interpreter, InterpreterError, Result, Stdlib};
use crate::linter::{BuiltInSub, HasQualifier, QualifiedName, TypeQualifier};
use crate::variant::Variant;

impl<S: Stdlib> Interpreter<S> {
    pub fn run_built_in_sub(&mut self, name: &BuiltInSub, pos: Location) -> Result<()> {
        match name {
            BuiltInSub::Print => {
                let mut print_args: Vec<String> = vec![];
                loop {
                    match self.context_mut().demand_sub().try_pop_front_unnamed() {
                        Some(v) => print_args.push(v.to_string()),
                        None => {
                            break;
                        }
                    }
                }
                self.stdlib.print(print_args);
                Ok(())
            }
            BuiltInSub::Environ => self.do_environ_sub(pos),
            BuiltInSub::Input => self.do_input(pos),
            BuiltInSub::System => panic!("Should have been handled at the IG level"),
        }
    }

    fn do_environ_sub(&mut self, pos: Location) -> Result<()> {
        match self.context_mut().demand_sub().pop_front_unnamed() {
            Variant::VString(arg_string_value) => {
                let parts: Vec<&str> = arg_string_value.split("=").collect();
                if parts.len() != 2 {
                    err("Invalid expression. Must be name=value.", pos)
                } else {
                    self.stdlib
                        .set_env_var(parts[0].to_string(), parts[1].to_string());
                    Ok(())
                }
            }
            _ => panic!("Type mismatch"),
        }
    }

    fn do_input(&mut self, pos: Location) -> Result<()> {
        loop {
            match &self.context_mut().demand_sub().pop_front_unnamed_arg() {
                Some(a) => match a {
                    Argument::ByRef(n) => {
                        self.do_input_one_var(a, n, pos)?;
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

    fn do_input_one_var(&mut self, a: &Argument, n: &QualifiedName, pos: Location) -> Result<()> {
        let raw_input: String = self
            .stdlib
            .input()
            .map_err(|e| InterpreterError::new_with_pos(e.to_string(), pos))?;
        let q: TypeQualifier = n.qualifier();
        let variable_value = match q {
            TypeQualifier::BangSingle => Variant::from(
                parse_single_input(raw_input)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?,
            ),
            TypeQualifier::DollarString => Variant::from(raw_input),
            TypeQualifier::PercentInteger => Variant::from(
                parse_int_input(raw_input).map_err(|e| InterpreterError::new_with_pos(e, pos))?,
            ),
            _ => unimplemented!(),
        };
        self.context_mut()
            .demand_sub()
            .set_value_to_popped_arg(a, variable_value);
        Ok(())
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
    use super::super::test_utils::*;
    use crate::assert_linter_err;
    use crate::linter::LinterError;

    #[test]
    fn test_sub_call_system_no_args_allowed() {
        assert_linter_err!("SYSTEM 42", LinterError::ArgumentCountMismatch, 1, 1);
    }
}
