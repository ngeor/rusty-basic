use super::error::*;
use super::post_conversion_linter::*;
use super::types::*;
use crate::parser::QualifiedName;

pub struct ForNextCounterMatch;

impl PostConversionLinter for ForNextCounterMatch {
    fn visit_for_loop(&self, f: &ForLoopNode) -> Result<(), Error> {
        self.visit_statement_nodes(&f.statements)?;

        // for and next counters must match
        match &f.next_counter {
            Some(n) => {
                let next_var_name: &QualifiedName = n.as_ref();
                if next_var_name == f.variable_name.as_ref() {
                    Ok(())
                } else {
                    err_l(LinterError::NextWithoutFor, n)
                }
            }
            None => Ok(()),
        }
    }
}
