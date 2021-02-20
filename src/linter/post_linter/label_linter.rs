use super::post_conversion_linter::*;
use crate::common::*;
use crate::parser::{
    BareName, FunctionImplementation, ProgramNode, QualifiedName, ResumeOption, SubImplementation,
};
use std::collections::{HashMap, HashSet};

#[derive(Default)]
pub struct LabelLinter {
    collecting: bool,
    labels: HashMap<LabelOwner, HashSet<CaseInsensitiveString>>,
    current_label_owner: LabelOwner,
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum LabelOwner {
    Global,
    Sub(BareName),
    Function(QualifiedName),
}

impl Default for LabelOwner {
    fn default() -> Self {
        Self::Global
    }
}

impl LabelLinter {
    fn do_visit_program(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        p.iter()
            .map(|t| self.visit_top_level_token_node(t))
            .collect()
    }

    fn contains_label_in_any_scope(&self, label: &CaseInsensitiveString) -> bool {
        for v in self.labels.values() {
            if v.contains(label) {
                return true;
            }
        }
        false
    }
}

impl PostConversionLinter for LabelLinter {
    fn visit_program(&mut self, p: &ProgramNode) -> Result<(), QErrorNode> {
        self.labels.insert(LabelOwner::Global, HashSet::new());
        self.collecting = true;
        self.do_visit_program(p)?;
        self.collecting = false;
        self.do_visit_program(p)
    }

    fn visit_function_implementation(
        &mut self,
        f: &FunctionImplementation,
    ) -> Result<(), QErrorNode> {
        let Locatable {
            element: function_name,
            ..
        } = f.name.clone();
        self.current_label_owner = LabelOwner::Function(function_name.demand_qualified());
        if self.collecting {
            self.labels
                .insert(self.current_label_owner.clone(), HashSet::new());
        }
        let result = self.visit_statement_nodes(&f.body);
        self.current_label_owner = LabelOwner::Global;
        result
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) -> Result<(), QErrorNode> {
        self.current_label_owner = LabelOwner::Sub(s.name.element.clone());
        if self.collecting {
            self.labels
                .insert(self.current_label_owner.clone(), HashSet::new());
        }
        let result = self.visit_statement_nodes(&s.body);
        self.current_label_owner = LabelOwner::Global;
        result
    }

    fn visit_error_handler(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        if self.collecting {
            return Ok(());
        }

        let labels = self.labels.get(&self.current_label_owner).unwrap();
        if labels.contains(label) {
            Ok(())
        } else {
            err_no_pos(QError::LabelNotDefined)
        }
    }

    fn visit_label(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        if !self.collecting {
            return Ok(());
        }

        if self.contains_label_in_any_scope(label) {
            err_no_pos(QError::DuplicateLabel)
        } else {
            let labels = self.labels.get_mut(&self.current_label_owner).unwrap();
            labels.insert(label.clone());
            Ok(())
        }
    }

    fn visit_go_to(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.visit_error_handler(label)
    }

    fn visit_go_sub(&mut self, label: &CaseInsensitiveString) -> Result<(), QErrorNode> {
        self.visit_error_handler(label)
    }

    fn visit_return(
        &mut self,
        opt_label: Option<&CaseInsensitiveString>,
    ) -> Result<(), QErrorNode> {
        match opt_label {
            Some(label) => self.visit_error_handler(label),
            _ => Ok(()),
        }
    }

    fn visit_resume(&mut self, resume_option: &ResumeOption) -> Result<(), QErrorNode> {
        if let ResumeOption::Label(label) = resume_option {
            self.visit_error_handler(label)
        } else {
            Ok(())
        }
    }
}
