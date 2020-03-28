use super::context::ReadWriteContext;
use super::context::Variant;
use super::*;
use crate::common::Result;
use crate::parser::{Block, Expression, NameWithTypeQualifier};
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn for_loop(
        &mut self,
        i: &NameWithTypeQualifier,
        a: &Expression,
        b: &Expression,
        statements: &Block,
    ) -> Result<()> {
        let mut start = self.evaluate_expression_as_int(a)?;
        let mut stop = self.evaluate_expression_as_int(b)?;
        while start <= stop {
            let counter_var_name = i.name();
            self.set_variable(counter_var_name, Variant::VNumber(start))?;
            self.statements(&statements)?;

            start += 1;
            stop = self.evaluate_expression_as_int(b)?;
        }

        Ok(())
    }
}
