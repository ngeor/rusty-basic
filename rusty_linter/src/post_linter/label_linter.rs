use super::post_conversion_linter::*;
use crate::core::LintResult;
use crate::core::{LintError, LintErrorPos};
use rusty_common::*;
use rusty_parser::*;
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct LabelLinter {
    labels: HashMap<LabelOwner, HashSet<CaseInsensitiveString>>,
    current_label_owner: LabelOwner,
}

#[derive(Default)]
struct LabelCollector {
    labels: HashMap<LabelOwner, HashSet<CaseInsensitiveString>>,
    current_label_owner: LabelOwner,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum LabelOwner {
    Global,
    // TODO prevent clone, store a reference here
    Sub(BareName),
    Function(QualifiedName),
}

impl Default for LabelOwner {
    fn default() -> Self {
        Self::Global
    }
}

impl LabelLinter {
    fn ensure_label_is_defined(
        &self,
        label: &CaseInsensitiveString,
        label_owner: &LabelOwner,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        self.labels
            .get(label_owner)
            .and_then(|set| set.get(label))
            .map(|_| ())
            .ok_or(LintError::LabelNotDefined)
            .with_err_at(&pos)
    }

    fn ensure_is_global_label(
        &self,
        label: &CaseInsensitiveString,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        self.ensure_label_is_defined(label, &LabelOwner::Global, pos)
    }

    fn ensure_is_current_label(
        &self,
        label: &CaseInsensitiveString,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        self.ensure_label_is_defined(label, &self.current_label_owner, pos)
    }
}

impl PostConversionLinter for LabelLinter {
    fn visit_program(&mut self, p: &Program) -> Result<(), LintErrorPos> {
        let mut collector = LabelCollector::default();
        collector.visit_program(p)?;
        self.labels = collector.labels;
        self.visit_global_statements(p)
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), LintErrorPos> {
        self.on_function(f)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.on_sub(s)
    }

    fn visit_on_error(
        &mut self,
        on_error_option: &OnErrorOption,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        match on_error_option {
            OnErrorOption::Label(label) => self.ensure_is_global_label(label, pos),
            _ => Ok(()),
        }
    }

    fn visit_go_to(
        &mut self,
        label: &CaseInsensitiveString,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        self.ensure_is_current_label(label, pos)
    }

    fn visit_go_sub(
        &mut self,
        label: &CaseInsensitiveString,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        self.ensure_is_current_label(label, pos)
    }

    fn visit_resume(
        &mut self,
        resume_option: &ResumeOption,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        if let ResumeOption::Label(label) = resume_option {
            self.ensure_is_current_label(label, pos)
        } else {
            Ok(())
        }
    }

    fn visit_return(
        &mut self,
        opt_label: Option<&CaseInsensitiveString>,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        match opt_label {
            Some(label) => self.ensure_is_current_label(label, pos),
            _ => Ok(()),
        }
    }
}

impl PostConversionLinter for LabelCollector {
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), LintErrorPos> {
        self.on_function(f)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.on_sub(s)
    }

    fn visit_label(
        &mut self,
        label: &CaseInsensitiveString,
        pos: Position,
    ) -> Result<(), LintErrorPos> {
        if self.labels.values().any(|s| s.contains(label)) {
            // labels need to be unique across all scopes
            Err(LintError::DuplicateLabel.at_pos(pos))
        } else {
            // TODO prevent clone for key and value?
            self.labels
                .entry(self.current_label_owner.clone())
                .or_insert_with(HashSet::new)
                .insert(label.clone());
            Ok(())
        }
    }
}

trait LabelOwnerHolder: PostConversionLinter {
    fn set_label_owner(&mut self, label_owner: LabelOwner);

    fn on_function(&mut self, f: &FunctionImplementation) -> Result<(), LintErrorPos> {
        let Positioned {
            element: function_name,
            ..
        } = f.name.clone();
        self.set_label_owner(LabelOwner::Function(function_name.demand_qualified()));
        self.visit_statements(&f.body)?;
        self.set_label_owner(LabelOwner::Global);
        Ok(())
    }

    fn on_sub(&mut self, s: &SubImplementation) -> Result<(), LintErrorPos> {
        self.set_label_owner(LabelOwner::Sub(s.name.element.clone()));
        self.visit_statements(&s.body)?;
        self.set_label_owner(LabelOwner::Global);
        Ok(())
    }
}

impl LabelOwnerHolder for LabelLinter {
    fn set_label_owner(&mut self, label_owner: LabelOwner) {
        self.current_label_owner = label_owner;
    }
}

impl LabelOwnerHolder for LabelCollector {
    fn set_label_owner(&mut self, label_owner: LabelOwner) {
        self.current_label_owner = label_owner;
    }
}
