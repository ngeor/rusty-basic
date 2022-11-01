use super::post_conversion_linter::*;
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
    ) -> Result<(), QErrorNode> {
        self.labels
            .get(label_owner)
            .and_then(|set| set.get(label))
            .map(|_| ())
            .ok_or_else(|| QError::LabelNotDefined)
            .with_err_no_pos()
    }

    fn ensure_is_global_label(&self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.ensure_label_is_defined(label, &LabelOwner::Global)
    }

    fn ensure_is_current_label(&self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.ensure_label_is_defined(label, &self.current_label_owner)
    }
}

impl PostConversionLinter for LabelLinter {
    fn visit_program(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        let mut collector = LabelCollector::default();
        collector.visit_program(p)?;
        self.labels = collector.labels;
        self.visit_top_level_token_nodes(p)
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.on_function(f)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.on_sub(s)
    }

    fn visit_on_error(&mut self, on_error_option: &OnErrorOption) -> Result<(), QErrorNode> {
        match on_error_option {
            OnErrorOption::Label(label) => self.ensure_is_global_label(label),
            _ => Ok(()),
        }
    }

    fn visit_go_to(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.ensure_is_current_label(label)
    }

    fn visit_go_sub(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.ensure_is_current_label(label)
    }

    fn visit_resume(&mut self, resume_option: &ResumeOption) -> Result<(), QErrorNode> {
        if let ResumeOption::Label(label) = resume_option {
            self.ensure_is_current_label(label)
        } else {
            Ok(())
        }
    }

    fn visit_return(
        &mut self,
        opt_label: Option<&CaseInsensitiveString>,
    ) -> Result<(), QErrorNode> {
        match opt_label {
            Some(label) => self.ensure_is_current_label(label),
            _ => Ok(()),
        }
    }
}

impl PostConversionLinter for LabelCollector {
    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        self.on_function(f)
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.on_sub(s)
    }

    fn visit_label(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        if self.labels.values().any(|s| s.contains(label)) {
            // labels need to be unique across all scopes
            Err(QError::DuplicateLabel).with_err_no_pos()
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

    fn on_function(&mut self, f: &FunctionImplementation) -> Result<(), QErrorNode> {
        let Locatable {
            element: function_name,
            ..
        } = f.name.clone();
        self.set_label_owner(LabelOwner::Function(function_name.demand_qualified()));
        self.visit_statement_nodes(&f.body)?;
        self.set_label_owner(LabelOwner::Global);
        Ok(())
    }

    fn on_sub(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.set_label_owner(LabelOwner::Sub(s.name.element.clone()));
        self.visit_statement_nodes(&s.body)?;
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
