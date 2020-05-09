use super::error::*;
use super::post_conversion_linter::*;
use crate::common::*;
use std::cell::RefCell;
use std::collections::HashSet;

pub struct LabelLinter {
    // implemented as RefCell for inner mutability
    labels: RefCell<HashSet<CaseInsensitiveString>>,

    // pass == 0, collecting labels
    // pass == 1, ensuring all labels exist
    collecting: bool,
}

impl LabelLinter {
    pub fn new() -> Self {
        Self {
            labels: RefCell::new(HashSet::new()),
            collecting: true,
        }
    }

    pub fn switch_to_validating_mode(&mut self) {
        if self.collecting {
            self.collecting = false
        } else {
            panic!("Invalid existing mode")
        }
    }
}

impl PostConversionLinter for LabelLinter {
    fn visit_error_handler(&self, label: &CaseInsensitiveString) -> Result<(), Error> {
        if self.collecting || self.labels.borrow().contains(label) {
            Ok(())
        } else {
            err_no_pos(LinterError::LabelNotFound)
        }
    }

    fn visit_label(&self, label: &CaseInsensitiveString) -> Result<(), Error> {
        if self.collecting {
            if self.labels.borrow().contains(label) {
                err_no_pos(LinterError::DuplicateDefinition)
            } else {
                self.labels.borrow_mut().insert(label.clone());
                Ok(())
            }
        } else {
            Ok(())
        }
    }

    fn visit_go_to(&self, label: &CaseInsensitiveString) -> Result<(), Error> {
        if self.collecting || self.labels.borrow().contains(label) {
            Ok(())
        } else {
            err_no_pos(LinterError::LabelNotFound)
        }
    }
}
