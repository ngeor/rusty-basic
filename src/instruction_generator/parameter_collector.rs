use crate::common::{Locatable, StripLocation};
use crate::parser::{
    BareName, FunctionImplementation, ParamName, ProgramNode, SubImplementation, TopLevelToken,
};
use std::collections::HashMap;

type ParamMap = HashMap<BareName, Vec<ParamName>>;

#[derive(Default)]
pub struct ParameterCollector {
    functions: ParamMap,
    subs: ParamMap,
}

impl ParameterCollector {
    pub fn visit(&mut self, program: &ProgramNode) {
        for Locatable { element, .. } in program {
            self.visit_top_level_token(element);
        }
    }

    fn visit_top_level_token(&mut self, top_level_token: &TopLevelToken) {
        match top_level_token {
            TopLevelToken::FunctionImplementation(f) => {
                self.visit_function_implementation(f);
            }
            TopLevelToken::SubImplementation(s) => {
                self.visit_sub_implementation(s);
            }
            _ => {}
        }
    }

    fn visit_function_implementation(&mut self, f: &FunctionImplementation) {
        self.functions.insert(
            f.name.bare_name().clone(),
            f.params.clone().strip_location(),
        );
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) {
        self.subs
            .insert(s.name.element.clone(), s.params.clone().strip_location());
    }
}

pub struct SubProgramParameters {
    functions: ParamMap,
    subs: ParamMap,
}

impl SubProgramParameters {
    pub fn new(functions: ParamMap, subs: ParamMap) -> Self {
        Self { functions, subs }
    }

    pub fn get_function_parameters(&self, name: &BareName) -> &Vec<ParamName> {
        self.functions.get(name).unwrap()
    }

    pub fn get_sub_parameters(&self, name: &BareName) -> &Vec<ParamName> {
        self.subs.get(name).unwrap()
    }
}

impl From<ParameterCollector> for SubProgramParameters {
    fn from(parameter_collector: ParameterCollector) -> Self {
        let ParameterCollector { functions, subs } = parameter_collector;
        SubProgramParameters::new(functions, subs)
    }
}
