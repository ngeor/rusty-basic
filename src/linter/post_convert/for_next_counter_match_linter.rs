use super::post_conversion_linter::*;
use crate::common::*;
use crate::linter::types::*;

pub struct ForNextCounterMatch;

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), QErrorNode> {
        self.visit_statement_nodes(&f.statements)?;

        // for and next counters must match
        // TODO verify FOR variable is numeric and not string
        match &f.next_counter {
            Some(Locatable { element, pos }) => {
                if *element == f.variable_name {
                    Ok(())
                } else {
                    Err(QError::NextWithoutFor).with_err_at(*pos)
                }
            }
            None => Ok(()),
        }
    }
}
