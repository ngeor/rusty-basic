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
        let mut start = self.evaluate_expression(a)?;
        let mut stop = self.evaluate_expression(b)?;
        while start.cmp(&stop)? != std::cmp::Ordering::Greater {
            self.set_variable(counter_var_name, start.clone())?;
            self.statements(&statements)?;

            start = start.plus(&Variant::from(1))?;
            stop = self.evaluate_expression(b)?;
        }

        Ok(())
    }
}
