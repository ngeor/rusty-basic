use super::{InterpreterError, Result, Variant};
use crate::common::{CaseInsensitiveString, HasLocation, Location};
use crate::interpreter::casting::cast;
use crate::parser::{
    HasQualifier, Name, NameNode, QualifiedName, ResolveIntoRef, TypeQualifier, TypeResolver,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Default)]
pub struct Context<T: TypeResolver> {
    variables: HashMap<CaseInsensitiveString, HashMap<TypeQualifier, Variant>>,
    constants: HashMap<CaseInsensitiveString, Variant>,
    result_name: Option<QualifiedName>,
    parent: Option<Box<Context<T>>>,
    resolver: Rc<RefCell<T>>,
}

impl<T: TypeResolver> Context<T> {
    pub fn new(resolver: Rc<RefCell<T>>) -> Self {
        Context {
            variables: HashMap::new(),
            constants: HashMap::new(),
            result_name: None,
            parent: None,
            resolver: resolver,
        }
    }

    pub fn get_function_result(&self, pos: Location) -> Variant {
        match &self.result_name {
            Some(qualified_result_name) => {
                let opt_result: Option<&Variant> =
                    match self.variables.get(qualified_result_name.bare_name()) {
                        Some(inner_map) => inner_map.get(&qualified_result_name.qualifier()),
                        None => None,
                    };
                match opt_result {
                    Some(v) => v.clone(),
                    None => Variant::default_variant(qualified_result_name.qualifier()),
                }
            }
            None => panic!(format!("Not in a function context at {:?}", pos)),
        }
    }

    fn insert_variable(
        &mut self,
        name: QualifiedName,
        value: Variant,
        pos: Location,
    ) -> Result<()> {
        if Variant::default_variant(name.qualifier()).is_same_type(&value) {
            let (bare_name, qualifier) = name.consume();
            if self.constants.contains_key(&bare_name) {
                Err(InterpreterError::new_with_pos("Duplicate definition", pos))
            } else {
                match self.variables.get_mut(&bare_name) {
                    Some(inner_map) => {
                        inner_map.insert(qualifier, value);
                        Ok(())
                    }
                    None => {
                        let mut inner_map: HashMap<TypeQualifier, Variant> = HashMap::new();
                        inner_map.insert(qualifier, value);
                        self.variables.insert(bare_name, inner_map);
                        Ok(())
                    }
                }
            }
        } else {
            Err(InterpreterError::new_with_pos(
                format!("Type mismatch {} {}", name, value),
                pos,
            ))
        }
    }

    pub fn set_const(
        &mut self,
        name: CaseInsensitiveString,
        value: Variant,
        pos: Location,
    ) -> Result<()> {
        if self.variables.contains_key(&name) || self.constants.contains_key(&name) {
            Err(InterpreterError::new_with_pos("Duplicate definition", pos))
        } else {
            self.constants.insert(name, value);
            Ok(())
        }
    }

    pub fn push_function(self, result_name: QualifiedName) -> Self {
        Context {
            variables: HashMap::new(),
            constants: HashMap::new(),
            result_name: Some(result_name),
            resolver: Rc::clone(&self.resolver),
            parent: Some(Box::new(self)),
        }
    }

    pub fn push_sub(self) -> Self {
        Context {
            variables: HashMap::new(),
            constants: HashMap::new(),
            result_name: None,
            resolver: Rc::clone(&self.resolver),
            parent: Some(Box::new(self)),
        }
    }

    pub fn pop(self) -> Self {
        match self.parent {
            Some(x) => *x,
            None => panic!("Stack underflow"),
        }
    }

    fn get_result_name(&self) -> &Option<QualifiedName> {
        &self.result_name
    }

    pub fn get_or_default(&self, name_node: &NameNode) -> Result<Variant> {
        self._get(name_node, false).map(|opt_var| match opt_var {
            Some(v_ref) => v_ref.clone(),
            None => Variant::default_variant(name_node.resolve_into(&self.resolver)),
        })
    }

    pub fn get_const(&self, name_node: &NameNode) -> Result<&Variant> {
        self._get(name_node, true)
            .and_then(|opt_var| match opt_var {
                Some(v_ref) => Ok(v_ref),
                None => Err(InterpreterError::new_with_pos(
                    "Variable not defined",
                    name_node.location(),
                )),
            })
    }

    pub fn get(&self, name_node: &NameNode) -> Result<&Variant> {
        self._get(name_node, false)
            .and_then(|opt_var| match opt_var {
                Some(v_ref) => Ok(v_ref),
                None => Err(InterpreterError::new_with_pos(
                    "Variable not defined",
                    name_node.location(),
                )),
            })
    }

    fn _get(&self, name_node: &NameNode, only_constants: bool) -> Result<Option<&Variant>> {
        self.get_from_me(name_node, only_constants)
            .and_then(|opt_var| match opt_var {
                Some(v) => Ok(Some(v)),
                None => self.get_from_parent(name_node, only_constants),
            })
    }

    fn get_from_me(&self, name_node: &NameNode, only_constants: bool) -> Result<Option<&Variant>> {
        let name: &Name = name_node.as_ref();
        let pos = name_node.location();
        match name {
            Name::Bare(bare_name) => self.get_bare(bare_name, pos, only_constants),
            Name::Typed(q) => self.get_qualified(q, pos, only_constants),
        }
    }

    fn get_from_parent(
        &self,
        name_node: &NameNode,
        only_constants: bool,
    ) -> Result<Option<&Variant>> {
        if only_constants {
            Ok(None)
        } else {
            match &self.parent {
                Some(p) => p._get(name_node, only_constants),
                None => Ok(None),
            }
        }
    }

    fn get_bare(
        &self,
        bare_name: &CaseInsensitiveString,
        pos: Location,
        only_constants: bool,
    ) -> Result<Option<&Variant>> {
        match self.constants.get(bare_name) {
            Some(v) => Ok(Some(v)),
            None => match self.variables.get(bare_name) {
                Some(inner_map) => {
                    let qualifier: TypeQualifier = self.resolver.resolve(bare_name);
                    match inner_map.get(&qualifier) {
                        Some(v) => {
                            if only_constants {
                                Err(InterpreterError::new_with_pos("Invalid constant", pos))
                            } else {
                                Ok(Some(v))
                            }
                        }
                        None => Ok(None),
                    }
                }
                None => Ok(None),
            },
        }
    }

    fn get_qualified(
        &self,
        qualified_name: &QualifiedName,
        pos: Location,
        only_constants: bool,
    ) -> Result<Option<&Variant>> {
        let bare_name: &CaseInsensitiveString = qualified_name.bare_name();
        match self.constants.get(bare_name) {
            Some(v) => {
                if v.qualifier() == qualified_name.qualifier() {
                    Ok(Some(v))
                } else {
                    Err(InterpreterError::new_with_pos("Duplicate definition", pos))
                }
            }
            None => match self.variables.get(bare_name) {
                Some(inner_map) => match inner_map.get(&qualified_name.qualifier()) {
                    Some(v) => {
                        if only_constants {
                            Err(InterpreterError::new_with_pos("Invalid constant", pos))
                        } else {
                            Ok(Some(v))
                        }
                    }
                    None => Ok(None),
                },
                None => Ok(None),
            },
        }
    }
}

pub trait VariableSetter<T> {
    fn set(&mut self, variable_name: T, variable_value: Variant) -> Result<()>;
}

impl<T: TypeResolver> VariableSetter<NameNode> for Context<T> {
    fn set(&mut self, variable_name: NameNode, variable_value: Variant) -> Result<()> {
        let (name, pos) = variable_name.consume();
        match name {
            Name::Bare(bare_name) => for_bare::set(self, bare_name, pos, variable_value),
            Name::Typed(q) => for_typed::set(self, q, pos, variable_value),
        }
    }
}

impl<T: VariableSetter<NameNode>> VariableSetter<&NameNode> for T {
    fn set(&mut self, variable_name: &NameNode, variable_value: Variant) -> Result<()> {
        self.set(variable_name.clone(), variable_value)
    }
}

mod for_bare {
    use super::{cast_insert, Context, QualifiedName, Result, TypeResolver, Variant};
    use crate::common::{CaseInsensitiveString, Location};

    pub fn set<T: TypeResolver>(
        context: &mut Context<T>,
        name: CaseInsensitiveString,
        pos: Location,
        variable_value: Variant,
    ) -> Result<()> {
        match context.get_result_name() {
            Some(result_name) => {
                if result_name.bare_name() != &name {
                    // different names, it does not match with the result name
                    resolve_and_set(context, name, pos, variable_value)
                } else {
                    // names match
                    // promote the bare name node to a qualified
                    let result_name_copy = result_name.clone();
                    cast_insert(context, result_name_copy, variable_value, pos)
                }
            }
            _ => resolve_and_set(context, name, pos, variable_value),
        }
    }

    fn resolve_and_set<T: TypeResolver>(
        context: &mut Context<T>,
        name: CaseInsensitiveString,
        pos: Location,
        value: Variant,
    ) -> Result<()> {
        let effective_type_qualifier = context.resolver.resolve(&name);
        let qualified_name = QualifiedName::new(name, effective_type_qualifier);
        cast_insert(context, qualified_name, value, pos)
    }
}

fn cast_insert<T: TypeResolver>(
    context: &mut Context<T>,
    name: QualifiedName,
    value: Variant,
    pos: Location,
) -> Result<()> {
    cast(value, name.qualifier())
        .map_err(|e| InterpreterError::new_with_pos(e, pos))
        .and_then(|casted| context.insert_variable(name, casted, pos))
}

mod for_typed {
    use super::{cast_insert, Context, InterpreterError, Result, TypeResolver, Variant};
    use crate::common::Location;
    use crate::parser::{HasQualifier, QualifiedName};

    pub fn set<T: TypeResolver>(
        context: &mut Context<T>,
        qualified_name: QualifiedName,
        pos: Location,
        variable_value: Variant,
    ) -> Result<()> {
        // make sure that if the name matches the function name then the type matches too
        match context.get_result_name() {
            Some(result_name) => {
                if result_name.bare_name() != qualified_name.bare_name() {
                    // different names, it does not match with the result name
                    cast_insert(context, qualified_name, variable_value, pos)
                } else {
                    // names match
                    if qualified_name.qualifier() == result_name.qualifier() {
                        cast_insert(context, qualified_name, variable_value, pos)
                    } else {
                        Err(InterpreterError::new_with_pos("Duplicate definition", pos))
                    }
                }
            }
            _ => cast_insert(context, qualified_name, variable_value, pos),
        }
    }
}
