use crate::parser::Block;
use crate::common::Result;
use std::collections::HashMap;

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: String,
}

impl FunctionDeclaration {
    pub fn new(name: String) -> FunctionDeclaration {
        FunctionDeclaration { name }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    pub name: String,
    pub parameters: Vec<String>,
    pub block: Block,
}

impl FunctionImplementation {
    pub fn new(name: String, parameters: Vec<String>, block: Block) -> FunctionImplementation {
        FunctionImplementation { name, parameters, block }
    }
}

/// A function context
#[derive(Debug)]
pub struct FunctionContext {
    function_declaration_map: HashMap<String, FunctionDeclaration>,
    function_implementation_map: HashMap<String, FunctionImplementation>,
}

impl FunctionContext {
    pub fn new() -> FunctionContext {
        FunctionContext {
            function_declaration_map: HashMap::new(),
            function_implementation_map: HashMap::new(),
        }
    }

    pub fn add_function_declaration<S: AsRef<str>>(&mut self, name: S) -> Result<()> {
        self.function_declaration_map.insert(
            name.as_ref().to_string(),
            FunctionDeclaration::new(name.as_ref().to_string()),
        );
        Ok(())
    }

    pub fn add_function_implementation<S: AsRef<str>>(
        &mut self,
        name: S,
        parameters: Vec<S>,
        block: Block,
    ) -> Result<()> {
        let owned_parameters: Vec<String> = parameters.iter().map(|x| x.as_ref().to_string()).collect();
        self.function_implementation_map.insert(
            name.as_ref().to_string(),
            FunctionImplementation::new(
                name.as_ref().to_string(),
                owned_parameters,
                block
            ),
        );
        Ok(())
    }

    pub fn get_function_declarations(&self) -> std::collections::hash_map::Keys<String, FunctionDeclaration> {
        self.function_declaration_map.keys()
    }

    pub fn get_function_implementation<S: AsRef<str>>(&self, name: S) -> Option<FunctionImplementation> {
        match self.function_implementation_map.get(name.as_ref()) {
            Some(f) => Some(f.clone()),
            None => None
        }
    }
}
