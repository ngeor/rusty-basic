use std::collections::HashMap;

use rusty_common::Positioned;
use rusty_linter::ScopeName;
use rusty_parser::*;

/// Holds information about a subprogram that is needed at runtime.
pub struct SubprogramInfo {
    /// The parameters of a subprogram.
    pub params: Vec<Parameter>,

    /// Specifies if the subprogram is static. Static subprograms preserve
    /// their variables between calls.
    pub is_static: bool,
}

impl SubprogramInfo {
    fn new<T>(subprogram_implementation: &SubprogramImplementation<T>) -> Self {
        let mut params: Vec<Parameter> = vec![];
        for Positioned { element, .. } in &subprogram_implementation.params {
            params.push(element.clone());
        }
        let is_static = subprogram_implementation.is_static;
        Self { params, is_static }
    }
}

#[derive(Default)]
pub struct SubprogramInfoCollector {
    map: HashMap<ScopeName, SubprogramInfo>,
}

impl SubprogramInfoCollector {
    pub fn visit(&mut self, program: &Program) {
        for Positioned { element, .. } in program {
            self.visit_global_statement(element);
        }
    }

    fn visit_global_statement(&mut self, global_statement: &GlobalStatement) {
        match global_statement {
            GlobalStatement::FunctionImplementation(f) => {
                self.visit_function_implementation(f);
            }
            GlobalStatement::SubImplementation(s) => {
                self.visit_sub_implementation(s);
            }
            _ => {}
        }
    }

    fn visit_function_implementation(&mut self, f: &FunctionImplementation) {
        let function_name = f.name.element.clone().demand_qualified();
        let scope_name = ScopeName::Function(function_name);
        self.map.insert(scope_name, SubprogramInfo::new(f));
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) {
        let sub_name = s.name.element.clone();
        let scope_name = ScopeName::Sub(sub_name);
        self.map.insert(scope_name, SubprogramInfo::new(s));
    }
}

pub struct SubprogramInfoRepository {
    map: HashMap<ScopeName, SubprogramInfo>,
}

impl SubprogramInfoRepository {
    pub fn new(map: HashMap<ScopeName, SubprogramInfo>) -> Self {
        Self { map }
    }

    pub fn get_subprogram_info(&self, scope_name: &ScopeName) -> &SubprogramInfo {
        self.map.get(scope_name).expect("Function/Sub not found")
    }
}

impl From<SubprogramInfoCollector> for SubprogramInfoRepository {
    fn from(subprogram_info_collector: SubprogramInfoCollector) -> Self {
        let SubprogramInfoCollector { map } = subprogram_info_collector;
        Self::new(map)
    }
}
