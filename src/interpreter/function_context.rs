use crate::common::Result;
use crate::parser::{Block, QName};
use std::collections::HashMap;

#[derive(Debug)]
pub struct FunctionDeclaration {
    pub name: QName,
    pub parameters: Vec<QName>,
}

impl FunctionDeclaration {
    pub fn new(name: QName, parameters: Vec<QName>) -> FunctionDeclaration {
        FunctionDeclaration { name, parameters }
    }
}

#[derive(Debug, Clone)]
pub struct FunctionImplementation {
    pub name: QName,
    pub parameters: Vec<QName>,
    pub block: Block,
}

impl FunctionImplementation {
    pub fn new(name: QName, parameters: Vec<QName>, block: Block) -> FunctionImplementation {
        FunctionImplementation {
            name,
            parameters,
            block,
        }
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

    pub fn add_function_declaration(&mut self, name: QName, parameters: Vec<QName>) -> Result<()> {
        self.function_declaration_map.insert(
            name.name().to_string(),
            FunctionDeclaration::new(name, parameters),
        );
        Ok(())
    }

    pub fn add_function_implementation(
        &mut self,
        name: QName,
        parameters: Vec<QName>,
        block: Block,
    ) -> Result<()> {
        self.function_implementation_map.insert(
            name.name().to_string(),
            FunctionImplementation::new(name, parameters, block),
        );
        Ok(())
    }

    pub fn get_function_declarations(
        &self,
    ) -> std::collections::hash_map::Keys<String, FunctionDeclaration> {
        self.function_declaration_map.keys()
    }

    pub fn get_function_implementation(&self, name: &String) -> Option<FunctionImplementation> {
        self.function_implementation_map
            .get(name)
            .map(|x| x.clone())
    }
}
