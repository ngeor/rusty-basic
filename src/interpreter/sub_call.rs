use super::context::ReadWriteContext;
use super::context::Variant;
use super::Interpreter;
use super::Stdlib;
use crate::common::Result;
use crate::parser::Expression;
use std::io::BufRead;

impl<T: BufRead, TStdlib: Stdlib> Interpreter<T, TStdlib> {
    pub fn sub_call(&mut self, name: &String, args: &Vec<Expression>) -> Result<()> {
        if name == "PRINT" {
            self._do_print(args)
        } else if name == "INPUT" {
            self._do_input(args)
        } else if name == "SYSTEM" {
            self.stdlib.system();
            Ok(())
        } else {
            Err(format!("Unknown sub {}", name))
        }
    }

    fn _do_print(&mut self, args: &Vec<Expression>) -> Result<()> {
        let mut strings: Vec<String> = vec![];
        for a in args {
            strings.push(self._do_print_map_arg(a)?);
        }
        self.stdlib.print(strings);
        Ok(())
    }

    fn _do_print_map_arg(&mut self, arg: &Expression) -> Result<String> {
        let evaluated = self.evaluate_expression_as_str(arg)?;
        Ok(format!("{}", evaluated))
    }

    fn _do_input(&mut self, args: &Vec<Expression>) -> Result<()> {
        for a in args {
            let variable_name = a.try_to_variable_name()?;
            let variable_value = self.stdlib.input()?;
            self.set_variable(variable_name, Variant::VString(variable_value))?;
        }
        Ok(())
    }
}
