use super::post_conversion_linter::*;
use super::types::*;
use crate::common::*;

pub struct ForNextCounterMatch;

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.statements)?;

        // for and next counters must match
        // TODO verify FOR variable is numeric and not string
        match &f.next_counter {
            Some(n) => {
                if *n == f.variable_name {
                    Ok(())
                } else {
                    // TODO fix pos
                    Err(QError::NextWithoutFor).with_err_no_pos()
                }
            }
            None => Ok(()),
        }
    }
}
