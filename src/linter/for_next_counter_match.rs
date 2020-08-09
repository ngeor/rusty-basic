use super::error::*;
use super::post_conversion_linter::*;
use super::types::*;
use crate::common::*;

pub struct ForNextCounterMatch;

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), LinterErrorNode> {
        self.visit_statement_nodes(&f.statements)?;

        // for and next counters must match
        // TODO verify FOR variable is numeric and not string
        match &f.next_counter {
            Some(n) => {
                let Locatable {
                    element: next_var_name,
                    pos,
                } = n;
                if next_var_name == f.variable_name.as_ref() {
                    Ok(())
                } else {
                    Err(LinterError::NextWithoutFor).with_err_at(pos)
                }
            }
            None => Ok(()),
        }
    }
}
