use crate::common::*;
use crate::interpreter::context::Argument;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{err_no_pos, Interpreter, InterpreterError, Result, Stdlib};
use crate::linter::QualifiedName;
use crate::parser::{HasQualifier, TypeQualifier};
use crate::variant::Variant;

impl<S: Stdlib> Interpreter<S> {
    pub fn line_input(&mut self) -> Result<()> {
        let mut is_first = true;
        let mut file_handle: FileHandle = FileHandle::default();
        let mut has_more = true;
        while has_more {
            let arg = &self.pop_unnamed_arg();
            match arg {
                Some(a) => match a {
                    Argument::ByVal(v) => {
                        if is_first && v.qualifier() == TypeQualifier::FileHandle {
                            file_handle = v.clone().demand_file_handle();
                        } else {
                            return err_no_pos("Argument type mismatch");
                        }
                    }
                    Argument::ByRef(n) => {
                        self.line_input_one(a, n, file_handle)?;
                    }
                },
                None => {
                    has_more = false;
                }
            }

            is_first = false;
        }
        Ok(())
    }

    fn line_input_one(
        &mut self,
        arg: &Argument,
        n: &QualifiedName,
        file_handle: FileHandle,
    ) -> Result<()> {
        if file_handle.is_valid() {
            self.line_input_one_file(arg, n, file_handle)
        } else {
            self.line_input_one_stdin(arg, n)
        }
    }

    fn line_input_one_file(
        &mut self,
        arg: &Argument,
        n: &QualifiedName,
        file_handle: FileHandle,
    ) -> Result<()> {
        let s = self
            .file_manager
            .read_line(file_handle)
            .map_err(|e| InterpreterError::new_no_pos(e.to_string()))?;
        let q: TypeQualifier = n.qualifier();
        match q {
            TypeQualifier::DollarString => self
                .context_mut()
                .demand_sub()
                .set_value_to_popped_arg(arg, Variant::VString(s))
                .map_err(|e| InterpreterError::new_no_pos(e)),
            _ => unimplemented!(),
        }
    }

    fn line_input_one_stdin(&mut self, _arg: &Argument, _n: &QualifiedName) -> Result<()> {
        unimplemented!()
    }
}
