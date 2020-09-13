use crate::common::CaseInsensitiveString;
use crate::instruction_generator::NamedRefParam;
use crate::linter::{
    ResolvedDeclaredName, ResolvedTypeDefinition, ResolvedUserDefinedTypes, UserDefinedName,
};
use crate::parser::{Name, QualifiedName, TypeQualifier};
use crate::variant::{DefaultForType, Variant};
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

//
// Argument
//

#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    ByVal(Variant),
    ByRef(ResolvedDeclaredName),
}

impl Argument {
    pub fn type_definition(&self) -> ResolvedTypeDefinition {
        match self {
            Self::ByVal(v) => v.type_definition(),
            Self::ByRef(r) => r.type_definition(),
        }
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
    fn insert(&mut self, name: K, value: Variant);
}

impl ConstantMapTrait<QualifiedName> for ConstantMap {
    fn get(&self, name: &QualifiedName) -> Option<&Variant> {
        self.0.get(name.as_ref())
    }

    fn insert(&mut self, name: QualifiedName, value: Variant) {
        let QualifiedName {
            name: bare_name, ..
        } = name;
        self.0.insert(bare_name, value);
    }
}

impl ConstantMapTrait<ResolvedDeclaredName> for ConstantMap {
    fn get(&self, name: &ResolvedDeclaredName) -> Option<&Variant> {
        self.0.get(name.as_ref())
    }

    fn insert(&mut self, name: ResolvedDeclaredName, value: Variant) {
        match name {
            ResolvedDeclaredName::BuiltIn(QualifiedName { name, .. }) => {
                self.0.insert(name, value);
            }
            _ => panic!("user defined type constant"),
        }
    }
}

// ========================================================
// VariableMap
// ========================================================

// BUG: misses info on A% A! A$ etc
#[derive(Debug)]
struct VariableMap(HashMap<Name, Variant>);

impl VariableMap {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn get(&self, resolved_declared_name: &ResolvedDeclaredName) -> Option<&Variant> {
        match resolved_declared_name {
            ResolvedDeclaredName::BuiltIn(qualified_name) => {
                let key: Name = Name::from(qualified_name.clone());
                self.0.get(&key)
            }
            _ => {
                let bare_names: Vec<CaseInsensitiveString> = resolved_declared_name.name_path();
                let bare_names_slice = &bare_names[..];
                let (first, rest) = bare_names_slice.split_first().expect("empty names!");
                let key: Name = Name::Bare(first.clone());
                if rest.is_empty() {
                    self.0.get(&key)
                } else {
                    match self.0.get(&key).expect("missing root variable") {
                        Variant::VUserDefined(user_defined_value) => {
                            user_defined_value.map().get_path(rest)
                        }
                        _ => panic!("cannot navigate simple variant"),
                    }
                }
            }
        }
    }

    pub fn insert(&mut self, resolved_declared_name: ResolvedDeclaredName, value: Variant) {
        match &resolved_declared_name {
            ResolvedDeclaredName::BuiltIn(qualified_name) => {
                let key: Name = Name::from(qualified_name.clone());
                self.0.insert(key, value);
            }
            _ => {
                let bare_names: Vec<CaseInsensitiveString> = resolved_declared_name.name_path();
                let bare_names_slice = &bare_names[..];
                let (first, rest) = bare_names_slice.split_first().expect("empty names!");
                let key: Name = Name::Bare(first.clone());
                if rest.is_empty() {
                    self.0.insert(key, value);
                } else {
                    match self.0.get_mut(&key).expect("missing root variable") {
                        Variant::VUserDefined(user_defined_value) => {
                            user_defined_value.map_mut().insert_path(rest, value);
                        }
                        _ => panic!("cannot navigate simple variant"),
                    }
                }
            }
        }
    }
}

// ========================================================
// ArgumentMap
// ========================================================

#[derive(Debug)]
struct ArgumentMap {
    named: HashMap<ResolvedDeclaredName, Argument>,
    name_order: VecDeque<ResolvedDeclaredName>,
}

impl ArgumentMap {
    pub fn new() -> Self {
        Self {
            named: HashMap::new(),
            name_order: VecDeque::new(),
        }
    }

    pub fn push_unnamed(&mut self, arg: Argument) {
        let dummy_name = format!("{}", self.name_order.len());
        let dummy_resolved_name = match arg.type_definition() {
            ResolvedTypeDefinition::BuiltIn(q) => {
                ResolvedDeclaredName::BuiltIn(QualifiedName::new(dummy_name.into(), q))
            }
            ResolvedTypeDefinition::String(_) => ResolvedDeclaredName::BuiltIn(QualifiedName::new(
                dummy_name.into(),
                TypeQualifier::DollarString,
            )),
            ResolvedTypeDefinition::UserDefined(u) => {
                ResolvedDeclaredName::UserDefined(UserDefinedName {
                    name: dummy_name.into(),
                    type_name: u,
                })
            }
        };
        self.insert(dummy_resolved_name, arg)
    }

    pub fn insert(&mut self, name: ResolvedDeclaredName, arg: Argument) {
        self.name_order.push_back(name.clone());
        match &arg {
            Argument::ByVal(v) => {
                self.named.insert(name, Argument::ByVal(v.clone()));
            }
            Argument::ByRef(_) => {
                self.named.insert(name, arg);
            }
        }
    }

    pub fn get_mut(&mut self, name: &ResolvedDeclaredName) -> Option<&mut Argument> {
        self.named.get_mut(name)
    }

    pub fn get(&self, name: &ResolvedDeclaredName) -> Option<&Argument> {
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
    user_defined_types: Rc<ResolvedUserDefinedTypes>,
}

impl RootContext {
    pub fn new(
        user_defined_types: Rc<ResolvedUserDefinedTypes>,
    ) -> Self {
        Self {
            variables: VariableMap::new(),
            constants: ConstantMap::new(),
            user_defined_types,
        }
    }

    pub fn get_r_value(&self, name: &ResolvedDeclaredName) -> Option<Variant> {
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

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) {
        self.constants.insert(name, value)
    }

    pub fn create_parameter(&mut self, name: ResolvedDeclaredName) -> Argument {
        match self.constants.get(&name) {
            Some(v) => Argument::ByVal(v.clone()),
            None => {
                match self.variables.get(&name) {
                    // ref pointing to var
                    Some(_) => Argument::ByRef(name),
                    None => {
                        // create the variable in this scope
                        // e.g. INPUT N
                        let q = match name.type_definition() {
                            ResolvedTypeDefinition::BuiltIn(q) => q,
                            _ => panic!("cannot implicitly create user defined variable"),
                        };
                        self.variables
                            .insert(name.clone(), Variant::default_variant(q));
                        Argument::ByRef(name)
                    }
                }
            }
        }
    }

    pub fn set_variable(&mut self, name: ResolvedDeclaredName, value: Variant) {
        // Arguments do not exist at root level. Create/Update a variable.
        self.variables.insert(name, value)
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
    pub fn push_back_unnamed_ref_parameter(&mut self, name: ResolvedDeclaredName) {
        let arg = self.parent.create_parameter(name);
        self.args.push_unnamed(arg)
    }

    pub fn push_back_unnamed_val_parameter(&mut self, value: Variant) {
        self.args.push_unnamed(Argument::ByVal(value))
    }

    pub fn set_named_ref_parameter(&mut self, named_ref_param: &NamedRefParam) {
        let NamedRefParam {
            argument_name,
            parameter_name,
        } = named_ref_param;
        let arg = self.parent.create_parameter(argument_name.clone());
        self.insert_next_argument(parameter_name, arg)
    }

    pub fn set_named_val_parameter(&mut self, param_name: &ResolvedDeclaredName, value: Variant) {
        self.insert_next_argument(param_name, Argument::ByVal(value))
    }

    fn insert_next_argument(&mut self, param_name: &ResolvedDeclaredName, arg: Argument) {
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
    fn set_variable_parent(&mut self, name: ResolvedDeclaredName, value: Variant) {
        self.parent.set_variable(name, value)
    }

    fn do_insert_variable(&mut self, name: ResolvedDeclaredName, value: Variant) {
        self.variables.insert(name, Argument::ByVal(value))
    }

    fn get_argument_mut(&mut self, name: &ResolvedDeclaredName) -> Option<&mut Argument> {
        self.variables.get_mut(name)
    }

    fn evaluate_argument(&self, arg: &Argument) -> Option<Variant> {
        match arg {
            Argument::ByVal(v) => Some(v.clone()),
            Argument::ByRef(n) => self.parent.get_r_value(n),
        }
    }

    fn get_variable(&self, name: &ResolvedDeclaredName) -> Option<&Argument> {
        self.variables.get(name)
    }

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) {
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

    pub fn set_value_to_popped_arg(&mut self, arg: &Argument, value: Variant) {
        match arg {
            Argument::ByVal(_) => panic!("Expected: variable"),
            Argument::ByRef(n) => {
                let q = n.clone(); // clone to break duplicate borrow
                self.set_variable_parent(q, value)
            }
        }
    }

    pub fn create_parameter(&mut self, name: ResolvedDeclaredName) -> Argument {
        match self.constants.get(&name) {
            Some(v) => Argument::ByVal(v.clone()),
            None => {
                // variable?
                match self.get_variable(&name) {
                    // ref pointing to var
                    Some(_) => Argument::ByRef(name),
                    None => {
                        // parent constant?
                        match self
                            .parent
                            .get_root()
                            .constants
                            .get(&name)
                            .map(|x| x.clone())
                        {
                            Some(v) => Argument::ByVal(v),
                            None => {
                                // create the variable in this scope
                                // e.g. INPUT N
                                let q = match name.type_definition() {
                                    ResolvedTypeDefinition::BuiltIn(q) => q,
                                    _ => panic!("cannot implicitly create user defined variable"),
                                };
                                self.do_insert_variable(name.clone(), Variant::default_variant(q));
                                Argument::ByRef(name)
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn set_variable(&mut self, name: ResolvedDeclaredName, value: Variant) {
        // if a parameter exists, set it (might be a ref)
        match self.get_argument_mut(&name) {
            Some(a) => {
                match a {
                    Argument::ByVal(_old_value) => {
                        *a = Argument::ByVal(value);
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

    pub fn get_r_value(&self, name: &ResolvedDeclaredName) -> Option<Variant> {
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
        user_defined_types: Rc<ResolvedUserDefinedTypes>,
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

    pub fn set_constant(&mut self, name: QualifiedName, value: Variant) {
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

    pub fn create_parameter(&mut self, name: ResolvedDeclaredName) -> Argument {
        match self {
            Self::Root(r) => r.create_parameter(name),
            Self::Sub(s) => s.create_parameter(name),
            Self::Args(a) => a.parent.create_parameter(name),
        }
    }

    pub fn set_variable(&mut self, name: ResolvedDeclaredName, value: Variant) {
        match self {
            Self::Root(r) => r.set_variable(name, value),
            Self::Sub(s) => s.set_variable(name, value),
            Self::Args(a) => a.parent.set_variable(name, value),
        }
    }

    pub fn get_r_value(&self, name: &ResolvedDeclaredName) -> Option<Variant> {
        match self {
            Self::Root(r) => r.get_r_value(name),
            Self::Args(a) => a.parent.get_r_value(name),
            Self::Sub(s) => s.get_r_value(name),
        }
    }
}
