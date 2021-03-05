use crate::common::Locatable;
use crate::parser::*;
use std::collections::HashMap;

pub struct SubprogramInfo {
    pub params: Vec<ParamName>,
    pub is_static: bool,
}

impl SubprogramInfo {
    fn new_from_subprogram_ref<T>(x: &SubprogramImplementation<T>) -> Self {
        let mut params: Vec<ParamName> = vec![];
        for Locatable { element, .. } in &x.params {
            params.push(element.clone());
        }
        let is_static = x.is_static;
        Self { params, is_static }
    }
}

#[derive(Default)]
pub struct SubprogramInfoCollector {
    functions: HashMap<QualifiedName, SubprogramInfo>,
    subs: HashMap<BareName, SubprogramInfo>,
}

impl SubprogramInfoCollector {
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
        let function_name = f.name.element.clone().demand_qualified();
        self.functions
            .insert(function_name, SubprogramInfo::new_from_subprogram_ref(f));
    }

    fn visit_sub_implementation(&mut self, s: &SubImplementation) {
        let sub_name = s.name.element.clone();
        self.subs
            .insert(sub_name, SubprogramInfo::new_from_subprogram_ref(s));
    }
}

pub struct SubprogramInfoRepository {
    // TODO use only one map HashMap<SubprogramName, SubprogramInfo>
    functions: HashMap<QualifiedName, SubprogramInfo>,
    subs: HashMap<BareName, SubprogramInfo>,
}

impl SubprogramInfoRepository {
    pub fn new(
        functions: HashMap<QualifiedName, SubprogramInfo>,
        subs: HashMap<BareName, SubprogramInfo>,
    ) -> Self {
        Self { functions, subs }
    }

    pub fn get_function_parameters(&self, name: &QualifiedName) -> &Vec<ParamName> {
        &self.functions.get(name).expect("Function not found").params
    }

    pub fn get_sub_parameters(&self, name: &BareName) -> &Vec<ParamName> {
        &self.get_sub_info(name).params
    }

    pub fn get_sub_info(&self, name: &BareName) -> &SubprogramInfo {
        self.subs.get(name).expect("Sub not found")
    }

    pub fn get_subprogram_info(&self, subprogram_name: &SubprogramName) -> &SubprogramInfo {
        match subprogram_name {
            SubprogramName::Sub(sub_name) => self.subs.get(sub_name).expect("Sub not found"),
            SubprogramName::Function(function_name) => self
                .functions
                .get(function_name)
                .expect("Function not found"),
        }
    }
}

impl From<SubprogramInfoCollector> for SubprogramInfoRepository {
    fn from(parameter_collector: SubprogramInfoCollector) -> Self {
        let SubprogramInfoCollector { functions, subs } = parameter_collector;
        SubprogramInfoRepository::new(functions, subs)
    }
}
