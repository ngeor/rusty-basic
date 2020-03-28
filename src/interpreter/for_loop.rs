use super::*;
use crate::common::Result;
use crate::parser::{Block, Expression, QName};
use std::io::BufRead;

impl<T: BufRead, S: Stdlib> Interpreter<T, S> {
    pub fn for_loop(
        &mut self,
        counter_var_name: &QName,
        a: &Expression,
        b: &Expression,
        statements: &Block,
    ) -> Result<()> {
        let mut start = self.evaluate_expression(a)?.to_int()?;
        let mut stop = self.evaluate_expression(b)?.to_int()?;
        while start <= stop {
            self.set_variable(counter_var_name, Variant::from(start))?;
            self.statements(&statements)?;

            start += 1;
            stop = self.evaluate_expression(b)?.to_int()?;
        }

        Ok(())
    }
}
