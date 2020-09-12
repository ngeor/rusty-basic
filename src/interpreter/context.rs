use crate::common::{CaseInsensitiveString, QError};
use crate::instruction_generator::NamedRefParam;
use crate::linter::casting;
use crate::linter::{
    HasQualifier, QualifiedName, ResolvedDeclaredName, ResolvedDeclaredNames,
    ResolvedTypeDefinition, ResolvedUserDefinedType, TypeQualifier,
};
use crate::variant::{DefaultForType, DefaultForTypes, UserDefinedValue, Variant, VariantMap};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

//
// Argument
//

#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    ByVal(Variant),
    ByRef(ResolvedDeclaredNames),
}

impl Argument {
    pub fn type_definition(&self) -> ResolvedTypeDefinition {
        match self {
            Self::ByVal(v) => v.type_definition(),
            Self::ByRef(r) => r.last().unwrap().type_definition.clone(),
        }
    }
}

//
// Cast
//

trait Cast<T> {
    fn cast(self, qualifier: T) -> Result<Self, QError>
    where
        Self: Sized;
}

impl Cast<TypeQualifier> for Variant {
    fn cast(self, qualifier: TypeQualifier) -> Result<Self, QError> {
        casting::cast(self, qualifier)
    }
}

impl Cast<ResolvedTypeDefinition> for Variant {
    fn cast(self, resolved_type_definition: ResolvedTypeDefinition) -> Result<Self, QError> {
        self.cast_for_type_definition(resolved_type_definition)
    }
}

// ========================================================
// ConstantMap
// ========================================================

#[derive(Debug)]
struct ConstantMap(HashMap<CaseInsensitiveString, Variant>);

impl ConstantMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
}

trait ConstantMapTrait<K> {
    fn get(&self, name: &K) -> Option<&Variant>;
    fn insert(&mut self, name: K, value: Variant) -> Result<(), QError>;
}

impl ConstantMapTrait<QualifiedName> for ConstantMap {
    fn get(&self, name: &QualifiedName) -> Option<&Variant> {
        self.0.get(name.as_ref())
    }

    fn insert(&mut self, name: QualifiedName, value: Variant) -> Result<(), QError> {
        let QualifiedName {
            name: bare_name,
            qualifier,
        } = name;
        self.0.insert(bare_name, value.cast(qualifier)?);
        Ok(())
    }
}

impl ConstantMapTrait<ResolvedDeclaredName> for ConstantMap {
    fn get(&self, name: &ResolvedDeclaredName) -> Option<&Variant> {
        self.0.get(name.as_ref())
    }

    fn insert(&mut self, name: ResolvedDeclaredName, value: Variant) -> Result<(), QError> {
        let ResolvedDeclaredName {
            name,
            type_definition,
        } = name;
        match type_definition {
            ResolvedTypeDefinition::BuiltIn(q) => {
                self.0.insert(name, value.cast(q)?);
                Ok(())
            }
            ResolvedTypeDefinition::UserDefined(_) => panic!("user defined type constant"),
        }
    }
}

impl ConstantMapTrait<ResolvedDeclaredNames> for ConstantMap {
    fn get(&self, name: &ResolvedDeclaredNames) -> Option<&Variant> {
        if name.len() == 1 {
            self.get(&name[0])
        } else {
            None
        }
    }

    fn insert(&mut self, name: ResolvedDeclaredNames, value: Variant) -> Result<(), QError> {
        panic!("user defined name constant")
    }
}

// ========================================================
// VariableMap
// ========================================================

#[derive(Debug)]
struct VariableMap(HashMap<CaseInsensitiveString, Variant>);

impl VariableMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, names: &ResolvedDeclaredNames) -> Option<&Variant> {
        let bare_names: Vec<CaseInsensitiveString> = names
            .iter()
            .map(|ResolvedDeclaredName { name, .. }| name.clone())
            .collect();
        let bare_names_slice = &bare_names[..];
        let (first, rest) = bare_names_slice.split_first().expect("empty names!");
        if rest.is_empty() {
            self.0.get(first)
        } else {
            match self.0.get(first).expect("missing root variable") {
                Variant::VUserDefined(user_defined_value) => {
                    user_defined_value.map().get_path(rest)
                }
                _ => panic!("cannot navigate simple variant"),
            }
        }
    }

    pub fn insert(&mut self, names: ResolvedDeclaredNames, value: Variant) -> Result<(), QError> {
        let bare_names: Vec<CaseInsensitiveString> = names
            .into_iter()
            .map(|ResolvedDeclaredName { name, .. }| name)
            .collect();
        let bare_names_slice = &bare_names[..];
        let (first, rest) = bare_names_slice.split_first().expect("empty names!");
        if rest.is_empty() {
            self.0.insert(first.clone(), value);
            Ok(())
        } else {
            match self.0.get_mut(first).expect("missing root variable") {
                Variant::VUserDefined(user_defined_value) => {
                    user_defined_value.map_mut().insert_path(rest, value);
                    Ok(())
                }
                _ => panic!("cannot navigate simple variant"),
            }
        }
    }
}

// ========================================================
// ArgumentMap
// ========================================================

#[derive(Debug)]
struct ArgumentMap {
    named: HashMap<ResolvedDeclaredNames, Argument>,
    name_order: VecDeque<ResolvedDeclaredNames>,
}

impl ArgumentMap {
    pub fn new() -> Self {
        Self {
            named: HashMap::new(),
            name_order: VecDeque::new(),
        }
    }

    pub fn push_unnamed(&mut self, arg: Argument) -> Result<(), QError> {
        let dummy_name = format!("{}", self.name_order.len());
        self.insert(
            ResolvedDeclaredName::single(dummy_name, arg.type_definition()),
            arg,
        )
    }

    pub fn insert(&mut self, name: ResolvedDeclaredNames, arg: Argument) -> Result<(), QError> {
        self.name_order.push_back(name.clone());
        match &arg {
            Argument::ByVal(v) => {
                let c = v
                    .clone()
                    .cast_for_type_definition(name.last().unwrap().type_definition.clone())?;
                self.named.insert(name, Argument::ByVal(c));
            }
            Argument::ByRef(_) => {
                self.named.insert(name, arg);
            }
        }
        Ok(())
    }

    pub fn get_mut(&mut self, name: &ResolvedDeclaredNames) -> Option<&mut Argument> {
        self.named.get_mut(name)
    }

    pub fn get(&self, name: &ResolvedDeclaredNames) -> Option<&Argument> {
        self.named.get(name)
    }

    pub fn pop_front(&mut self) -> Option<Argument> {
        match self.name_order.pop_front() {
            Some(name) => self.named.remove(&name),
            None => None,
        }
    }
}

//
// RootContext
//

#[derive(Debug)]
pub struct RootContext {
    variables: VariableMap,
    constants: ConstantMap,
    user_defined_types: Rc<HashMap<CaseInsensitiveString, ResolvedUserDefinedType>>,
}

impl RootContext {
    pub fn new(
        user_defined_types: Rc<HashMap<CaseInsensitiveString, ResolvedUserDefinedType>>,
    ) -> Self {
        Self {
            variables: VariableMap::new(),
            constants: ConstantMap::new(),
            user_defined_types,
        }
    }

    pub fn get_r_value(&self, name: &ResolvedDeclaredNames) -> Option<Variant> {
        // local constant?
        match self.constants.get(name) {
            Some(v) => Some(v.clone()),
            None => {
                // variable?
                match self.variables.get(name) {
                    Some(v) => Some(v.clone()),
                    None => None,
                }
            }
        }
    }

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) -> Result<(), QError> {
        self.constants.insert(name, value)
    }

    pub fn create_parameter(&mut self, name: ResolvedDeclaredNames) -> Argument {
        match self.constants.get(&name) {
            Some(v) => Argument::ByVal(v.clone()),
            None => {
                match self.variables.get(&name) {
                    // ref pointing to var
                    Some(_) => Argument::ByRef(name),
                    None => {
                        // create the variable in this scope
                        // e.g. INPUT N
                        let last = name.last().unwrap();
                        let type_definition = &last.type_definition;
                        let q = match type_definition {
                            ResolvedTypeDefinition::BuiltIn(q) => *q,
                            _ => panic!("cannot implicitly create user defined variable"),
                        };
                        self.variables
                            .insert(name.clone(), Variant::default_variant(q))
                            .expect("should work");
                        Argument::ByRef(name)
                    }
                }
            }
        }
    }

    pub fn set_variable(
        &mut self,
        name: ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        // Arguments do not exist at root level. Create/Update a variable.
        let x = value.cast_for_type_definition(name.last().unwrap().type_definition.clone())?;
        self.variables.insert(name, x)
    }
}

//
// ArgsContext (collecting arguments just before a function/sub call)
//

#[derive(Debug)]
pub struct ArgsContext {
    parent: Box<Context>,
    args: ArgumentMap,
}

impl ArgsContext {
    pub fn push_back_unnamed_ref_parameter(
        &mut self,
        name: ResolvedDeclaredNames,
    ) -> Result<(), QError> {
        let arg = self.parent.create_parameter(name)?;
        self.args.push_unnamed(arg)
    }

    pub fn push_back_unnamed_val_parameter(&mut self, value: Variant) -> Result<(), QError> {
        self.args.push_unnamed(Argument::ByVal(value))
    }

    pub fn set_named_ref_parameter(
        &mut self,
        named_ref_param: &NamedRefParam,
    ) -> Result<(), QError> {
        let NamedRefParam {
            argument_name,
            parameter_name,
        } = named_ref_param;
        let arg = self.parent.create_parameter(argument_name.clone())?;
        let param_names = vec![parameter_name.clone()];
        self.insert_next_argument(&param_names, arg)
    }

    pub fn set_named_val_parameter(
        &mut self,
        param_name: &ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        self.insert_next_argument(param_name, Argument::ByVal(value))
    }

    fn insert_next_argument(
        &mut self,
        param_name: &ResolvedDeclaredNames,
        arg: Argument,
    ) -> Result<(), QError> {
        self.args.insert(param_name.clone(), arg)
    }
}

//
// SubContext (inside a function or sub)
//

#[derive(Debug)]
pub struct SubContext {
    parent: Box<Context>,
    variables: ArgumentMap,
    constants: ConstantMap,
}

impl SubContext {
    fn set_variable_parent(
        &mut self,
        name: ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        self.parent.set_variable(name, value)
    }

    fn do_insert_variable(
        &mut self,
        name: ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        self.variables.insert(name, Argument::ByVal(value))
    }

    fn get_argument_mut(&mut self, name: &ResolvedDeclaredNames) -> Option<&mut Argument> {
        self.variables.get_mut(name)
    }

    fn evaluate_argument(&self, arg: &Argument) -> Option<Variant> {
        match arg {
            Argument::ByVal(v) => Some(v.clone()),
            Argument::ByRef(n) => self.parent.get_r_value(n),
        }
    }

    fn get_variable(&self, name: &ResolvedDeclaredNames) -> Option<&Argument> {
        self.variables.get(name)
    }

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) -> Result<(), QError> {
        self.constants.insert(name, value)
    }

    /// Pops the next unnamed argument, starting from the beginning.
    pub fn pop_unnamed_arg(&mut self) -> Option<Argument> {
        self.variables.pop_front()
    }

    /// Pops the value of the next unnamed argument, starting from the beginning.
    pub fn pop_unnamed_val(&mut self) -> Option<Variant> {
        match self.pop_unnamed_arg() {
            Some(arg) => self.evaluate_argument(&arg),
            None => None,
        }
    }

    pub fn set_value_to_popped_arg(
        &mut self,
        arg: &Argument,
        value: Variant,
    ) -> Result<(), QError> {
        match arg {
            Argument::ByVal(_) => panic!("Expected: variable"),
            Argument::ByRef(n) => {
                let q = n.clone(); // clone to break duplicate borrow
                self.set_variable_parent(q, value)
            }
        }
    }

    pub fn create_parameter(&mut self, name: ResolvedDeclaredNames) -> Result<Argument, QError> {
        match self.constants.get(&name) {
            Some(v) => Ok(Argument::ByVal(v.clone())),
            None => {
                // variable?
                match self.get_variable(&name) {
                    // ref pointing to var
                    Some(_) => Ok(Argument::ByRef(name)),
                    None => {
                        // parent constant?
                        match self
                            .parent
                            .get_root()
                            .constants
                            .get(&name)
                            .map(|x| x.clone())
                        {
                            Some(v) => Ok(Argument::ByVal(v)),
                            None => {
                                // create the variable in this scope
                                // e.g. INPUT N
                                let last = name.last().unwrap();
                                let type_definition = &last.type_definition;
                                let q = match type_definition {
                                    ResolvedTypeDefinition::BuiltIn(q) => *q,
                                    _ => panic!("cannot implicitly create user defined variable"),
                                };
                                self.do_insert_variable(name.clone(), Variant::default_variant(q))?;
                                Ok(Argument::ByRef(name))
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn set_variable(
        &mut self,
        name: ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        // if a parameter exists, set it (might be a ref)
        match self.get_argument_mut(&name) {
            Some(a) => {
                match a {
                    Argument::ByVal(_old_value) => {
                        *a = Argument::ByVal(value.cast_for_type_definition(
                            name.last().unwrap().type_definition.clone(),
                        )?);
                        Ok(())
                    }
                    Argument::ByRef(n) => {
                        let q = n.clone(); // clone needed to break duplicate borrow
                        self.set_variable_parent(q, value)
                    }
                }
            }
            None => {
                // A parameter does not exist. Create/Update a variable.
                self.do_insert_variable(name, value)
            }
        }
    }

    pub fn get_r_value(&self, name: &ResolvedDeclaredNames) -> Option<Variant> {
        // local constant?
        match self.constants.get(name) {
            Some(v) => Some(v.clone()),
            None => {
                // variable?
                match self.get_variable(name) {
                    Some(v) => self.evaluate_argument(v),
                    None => {
                        // top-level constant?
                        self.parent
                            .get_root()
                            .constants
                            .get(name)
                            .map(|x| x.clone())
                    }
                }
            }
        }
    }
}

//
// Context
//

#[derive(Debug)]
pub enum Context {
    Root(RootContext),
    Sub(SubContext),
    Args(ArgsContext),
}

impl Context {
    pub fn new(
        user_defined_types: Rc<HashMap<CaseInsensitiveString, ResolvedUserDefinedType>>,
    ) -> Self {
        Self::Root(RootContext::new(user_defined_types))
    }

    pub fn push_args_context(self) -> Self {
        Self::Args(ArgsContext {
            parent: Box::new(self),
            args: ArgumentMap::new(),
        })
    }

    pub fn swap_args_with_sub_context(self) -> Self {
        match self {
            Self::Args(a) => Self::Sub(SubContext {
                parent: a.parent,
                variables: a.args,
                constants: ConstantMap::new(),
            }),
            _ => panic!("Not in an args context"),
        }
    }

    pub fn pop(self) -> Self {
        match self {
            Self::Root(_) => panic!("Stack underflow"),
            Self::Sub(s) => *s.parent,
            Self::Args(_) => panic!("Did not finish args building"),
        }
    }

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) -> Result<(), QError> {
        match self {
            Self::Root(r) => r.set_constant(name, value),
            Self::Sub(s) => s.set_constant(name, value),
            _ => panic!("Not allowed in an arg context"),
        }
    }

    pub fn demand_args(&mut self) -> &mut ArgsContext {
        match self {
            Self::Args(a) => a,
            _ => panic!("Not in an args context"),
        }
    }

    pub fn demand_sub(&mut self) -> &mut SubContext {
        match self {
            Self::Sub(s) => s,
            _ => panic!("Not in a subprogram context"),
        }
    }

    pub fn get_root(&self) -> &RootContext {
        match self {
            Self::Root(r) => r,
            Self::Args(a) => a.parent.get_root(),
            Self::Sub(s) => s.parent.get_root(),
        }
    }

    pub fn create_parameter(&mut self, name: ResolvedDeclaredNames) -> Result<Argument, QError> {
        match self {
            Self::Root(r) => Ok(r.create_parameter(name)),
            Self::Sub(s) => s.create_parameter(name),
            Self::Args(a) => a.parent.create_parameter(name),
        }
    }

    pub fn set_variable(
        &mut self,
        name: ResolvedDeclaredNames,
        value: Variant,
    ) -> Result<(), QError> {
        match self {
            Self::Root(r) => r.set_variable(name, value),
            Self::Sub(s) => s.set_variable(name, value),
            Self::Args(a) => a.parent.set_variable(name, value),
        }
    }

    pub fn get_r_value(&self, name: &ResolvedDeclaredNames) -> Option<Variant> {
        match self {
            Self::Root(r) => r.get_r_value(name),
            Self::Args(a) => a.parent.get_r_value(name),
            Self::Sub(s) => s.get_r_value(name),
        }
    }
}
