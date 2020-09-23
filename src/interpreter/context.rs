use crate::common::CaseInsensitiveString;
use crate::linter::{DimName, ParamName, UserDefinedName, UserDefinedTypes};
use crate::parser::{QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::{HashMap, VecDeque};
use std::rc::Rc;

/*

Calling a sub

Example:

    Hello A, B

    SUB Hello(X, Y)
        Z = X + Y
    END SUB

Evaluate A in root context
Evaluate B in root context
Assign to X in sub context
Assign to Y in sub context
Call Hello



Example 2:


    Hello Add(A, B), Mul(A, B)

Evaluate first arg
    Evaluate A in root context
    Evaluate B in root context
    Assign to X in fn context
    Assign to Y in fn context
    Call Add
Evaluate second arg
    Evaluate A in root context
    Evaluate B in root context
    Assign to X in fn context
    Assign to Y in fn context
    Call Mul
Assign to X in sub context
Assign to Y in sub context
Call Hello

*/

// ========================================================
// Argument
// ========================================================

#[derive(Clone, Debug, PartialEq)]
pub enum Argument {
    ByVal(Variant),
    ByRef(DimName),
}

impl From<Variant> for Argument {
    fn from(v: Variant) -> Self {
        Self::ByVal(v)
    }
}

impl From<DimName> for Argument {
    fn from(n: DimName) -> Self {
        Self::ByRef(n)
    }
}

// ========================================================
// Arguments
// ========================================================

pub enum Arguments {
    Named(HashMap<ParamName, Argument>),
    Unnamed(VecDeque<Argument>),
}

impl Arguments {
    pub fn named() -> Self {
        Self::Named(HashMap::new())
    }

    pub fn unnamed() -> Self {
        Self::Unnamed(VecDeque::new())
    }

    pub fn get(&self, param_name: &ParamName) -> Option<&Argument> {
        match self {
            Self::Named(map) => map.get(param_name),
            Self::Unnamed(_) => None,
        }
    }

    pub fn get_mut(&mut self, param_name: &ParamName) -> Option<&mut Argument> {
        match self {
            Self::Named(map) => map.get_mut(param_name),
            Self::Unnamed(_) => None,
        }
    }

    pub fn push_unnamed<T>(&mut self, arg: T)
    where
        Argument: From<T>,
    {
        match self {
            Self::Named(_) => panic!("Cannot push unnamed in Arguments::Named"),
            Self::Unnamed(v) => v.push_back(arg.into()),
        }
    }

    pub fn push_named<T>(&mut self, parameter_name: ParamName, arg: T)
    where
        Argument: From<T>,
    {
        match self {
            Self::Named(m) => match m.insert(parameter_name, arg.into()) {
                Some(_) => panic!("Duplicate key!"),
                None => {}
            },
            Self::Unnamed(_) => panic!("Cannot push named in Arguments::Unnamed"),
        }
    }

    #[deprecated]
    pub fn pop_front(&mut self) -> Option<Argument> {
        match self {
            Self::Named(_) => None,
            Self::Unnamed(v) => v.pop_front(),
        }
    }

    pub fn iter(&self) -> std::collections::vec_deque::Iter<Argument> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.iter(),
        }
    }

    pub fn get_at(&self, index: usize) -> Option<&Argument> {
        match self {
            Self::Named(_) => panic!("Not supported for Arguments::Named"),
            Self::Unnamed(v) => v.get(index),
        }
    }
}

// ========================================================
// ArgumentsStack
// ========================================================

pub struct ArgumentsStack {
    stack: VecDeque<Arguments>,
}

impl ArgumentsStack {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    pub fn begin_collect_named_arguments(&mut self) {
        self.stack.push_back(Arguments::named());
    }

    pub fn begin_collect_unnamed_arguments(&mut self) {
        self.stack.push_back(Arguments::unnamed());
    }

    pub fn pop(&mut self) -> Arguments {
        self.stack
            .pop_back()
            .expect("Stack underflow collecting arguments!")
    }

    pub fn push_unnamed<T>(&mut self, arg: T)
    where
        Argument: From<T>,
    {
        self.demand().push_unnamed(arg);
    }

    pub fn push_named<T>(&mut self, param_name: ParamName, arg: T)
    where
        Argument: From<T>,
    {
        self.demand().push_named(param_name, arg);
    }

    fn demand(&mut self) -> &mut Arguments {
        self.stack.back_mut().expect("No arguments pushed!")
    }
}

// ========================================================
// Context
// ========================================================

pub struct Context {
    parent: Option<Box<Context>>,
    user_defined_types: Rc<UserDefinedTypes>,
    constants: HashMap<QualifiedName, Variant>,
    variables: HashMap<QualifiedName, Variant>,
    user_defined_type_variables: HashMap<CaseInsensitiveString, Variant>,

    /// Preparing arguments for the next call
    arguments_stack: ArgumentsStack,

    /// Got these parameters when this call started
    parameters: Arguments,
}

impl Context {
    pub fn new(user_defined_types: Rc<UserDefinedTypes>) -> Self {
        Self {
            parent: None,
            user_defined_types,
            constants: HashMap::new(),
            variables: HashMap::new(),
            user_defined_type_variables: HashMap::new(),
            arguments_stack: ArgumentsStack::new(),
            parameters: Arguments::unnamed(),
        }
    }

    pub fn pop(self) -> Self {
        match self.parent {
            Some(p) => *p,
            None => panic!("Stack underflow"),
        }
    }

    pub fn push(mut self) -> Self {
        let arguments: Arguments = self.arguments_stack.pop();
        Self {
            user_defined_types: Rc::clone(&self.user_defined_types),
            constants: HashMap::new(),
            variables: HashMap::new(),
            user_defined_type_variables: HashMap::new(),
            parameters: arguments,
            arguments_stack: ArgumentsStack::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn set_constant(&mut self, qualified_name: QualifiedName, value: Variant) {
        self.constants.insert(qualified_name, value);
    }

    pub fn set_variable(&mut self, dim_name: DimName, value: Variant) {
        // set a parameter or set a variable?
        match dim_name {
            DimName::BuiltIn(bare_name, qualifier) => {
                let p = ParamName::BuiltIn(bare_name.clone(), qualifier);
                match self.parameters.get_mut(&p) {
                    Some(arg) => match arg {
                        Argument::ByRef(name_in_parent) => {
                            let p = name_in_parent.clone();
                            self.parent
                                .as_mut()
                                .expect("should have parent")
                                .set_variable(p, value);
                        }
                        Argument::ByVal(_old_value) => {
                            *arg = Argument::ByVal(value);
                        }
                    },
                    None => {
                        self.variables
                            .insert(QualifiedName::new(bare_name, qualifier), value);
                    }
                }
            }
            DimName::String(name, _len) => {
                self.variables
                    .insert(QualifiedName::new(name, TypeQualifier::DollarString), value);
            }
            DimName::UserDefined(user_defined_name) => {
                let UserDefinedName { name, type_name } = user_defined_name.clone();
                let p = ParamName::UserDefined(name, type_name);
                match self.parameters.get_mut(&p) {
                    Some(arg) => match arg {
                        Argument::ByRef(name_in_parent) => {
                            let p = name_in_parent.clone();
                            self.parent
                                .as_mut()
                                .expect("should have parent")
                                .set_variable(p, value);
                        }
                        Argument::ByVal(_old_value) => {
                            *arg = Argument::ByVal(value);
                        }
                    },
                    None => {
                        self.user_defined_type_variables
                            .insert(user_defined_name.name, value);
                    }
                }
            }
            DimName::Many(user_defined_name, members) => {
                let UserDefinedName { name, type_name } = user_defined_name.clone();
                let p = ParamName::UserDefined(name, type_name);
                match self.parameters.get_mut(&p) {
                    Some(arg) => match arg {
                        Argument::ByRef(name_in_parent) => {
                            let p = name_in_parent.clone().append(members);
                            self.parent
                                .as_mut()
                                .expect("should have parent")
                                .set_variable(p, value);
                        }
                        Argument::ByVal(_old_value) => {
                            *arg = Argument::ByVal(value);
                        }
                    },
                    None => {
                        match self
                            .user_defined_type_variables
                            .get_mut(&user_defined_name.name)
                        {
                            Some(v) => match v {
                                Variant::VUserDefined(box_user_defined_type_value) => {
                                    let name_path = members.name_path();
                                    box_user_defined_type_value.insert_path(&name_path, value);
                                }
                                _ => unimplemented!(),
                            },
                            None => unimplemented!(),
                        }
                    }
                }
            }
        }
    }

    pub fn get_r_value(&self, name: &DimName) -> Option<Variant> {
        match self.try_get_r_value(name) {
            Some(v) => Some(v.clone()),
            None => {
                // create it
                match name {
                    DimName::BuiltIn(_, qualifier) => Some(qualifier.into()),
                    _ => unimplemented!(),
                }
            }
        }
    }

    // ========================================================
    // used to be ArgsContext
    // ========================================================

    pub fn arguments_stack(&mut self) -> &mut ArgumentsStack {
        &mut self.arguments_stack
    }

    // ========================================================
    // used to be subcontext
    // ========================================================

    pub fn set_value_to_popped_arg(&mut self, arg: &Argument, value: Variant) {
        match arg {
            Argument::ByVal(_) => panic!("Expected: variable"),
            Argument::ByRef(n) => self
                .parent
                .as_mut()
                .expect("should have parent")
                .set_variable(n.clone(), value),
        }
    }

    pub fn take_parameters(&mut self) -> Arguments {
        std::mem::replace(&mut self.parameters, Arguments::unnamed())
    }

    pub fn parameter_values(&self) -> ParameterValues {
        ParameterValues::new(self)
    }

    pub fn evaluate_parameter<'a>(&'a self, arg: &'a Argument) -> &'a Variant {
        match arg {
            Argument::ByRef(name_in_parent) => match &self.parent {
                Some(p) => p
                    .try_get_r_value(name_in_parent)
                    .expect("Should exist in parent"),
                None => panic!("Should have parent"),
            },
            Argument::ByVal(v) => v,
        }
    }

    // ========================================================
    // private
    // ========================================================

    fn try_get_r_value(&self, name: &DimName) -> Option<&Variant> {
        // get a constant or a local thing or a parent constant
        match name {
            DimName::BuiltIn(bare_name, qualifier) => {
                // is it a constant
                let qualified_name = &QualifiedName::new(bare_name.clone(), *qualifier);
                match self.constants.get(qualified_name) {
                    Some(v) => Some(v),
                    None => {
                        // is it a parameter
                        let p = ParamName::BuiltIn(bare_name.clone(), *qualifier);
                        match self.parameters.get(&p) {
                            Some(arg) => match arg {
                                Argument::ByRef(name_in_parent) => self
                                    .parent
                                    .as_ref()
                                    .expect("should have parent")
                                    .try_get_r_value(name_in_parent),
                                Argument::ByVal(v) => Some(v),
                            },
                            None => {
                                // is it a variable
                                match self.variables.get(qualified_name) {
                                    Some(v) => Some(v),
                                    None => {
                                        // is it a root constant
                                        self.get_root_const(qualified_name)
                                    }
                                }
                            }
                        }
                    }
                }
            }
            DimName::String(name, _len) => {
                let qualified_name = QualifiedName::new(name.clone(), TypeQualifier::DollarString);
                self.variables.get(&qualified_name)
            }
            DimName::UserDefined(user_defined_name) => {
                // is it a parameter
                let UserDefinedName { name, type_name } = user_defined_name.clone();
                let p = ParamName::UserDefined(name, type_name);
                match self.parameters.get(&p) {
                    Some(arg) => match arg {
                        Argument::ByRef(name_in_parent) => self
                            .parent
                            .as_ref()
                            .expect("should have parent")
                            .try_get_r_value(name_in_parent),
                        Argument::ByVal(v) => Some(v),
                    },
                    None => {
                        // is it a variable
                        match self
                            .user_defined_type_variables
                            .get(&user_defined_name.name)
                        {
                            Some(v) => Some(v),
                            None => {
                                // create it in this scope
                                unimplemented!()
                            }
                        }
                    }
                }
            }
            DimName::Many(user_defined_name, members) => {
                // is it a parameter
                let UserDefinedName { name, type_name } = user_defined_name.clone();
                let p = ParamName::UserDefined(name, type_name);
                match self.parameters.get(&p) {
                    Some(arg) => match arg {
                        Argument::ByRef(name_in_parent) => {
                            let p = name_in_parent.clone().append(members.clone());
                            self.parent
                                .as_ref()
                                .expect("should have parent")
                                .try_get_r_value(&p)
                        }
                        Argument::ByVal(v) => match v {
                            Variant::VUserDefined(box_user_defined_type_value) => {
                                let name_path = members.name_path();
                                box_user_defined_type_value.get_path(&name_path)
                            }
                            _ => unimplemented!(),
                        },
                    },
                    None => {
                        // is it a variable
                        match self
                            .user_defined_type_variables
                            .get(&user_defined_name.name)
                        {
                            Some(v) => match v {
                                Variant::VUserDefined(box_user_defined_type_value) => {
                                    let name_path = members.name_path();
                                    box_user_defined_type_value.get_path(&name_path)
                                }
                                _ => unimplemented!(),
                            },
                            None => {
                                // create it in this scope
                                unimplemented!()
                            }
                        }
                    }
                }
            }
        }
    }

    fn get_root_const(&self, name: &QualifiedName) -> Option<&Variant> {
        match &self.parent {
            Some(p) => {
                let mut context: &Self = p.as_ref();
                loop {
                    match &context.parent {
                        Some(p) => {
                            context = p;
                        }
                        None => {
                            break;
                        }
                    }
                }
                context.constants.get(name)
            }
            None => {
                // already at root context, therefore already checked
                None
            }
        }
    }
}

// ========================================================
// ParameterValues
// ========================================================

pub struct ParameterValues<'a> {
    context: &'a Context,
    parameters_iterator: std::collections::vec_deque::Iter<'a, Argument>,
}

impl<'a> ParameterValues<'a> {
    pub fn new(context: &'a Context) -> Self {
        ParameterValues {
            context,
            parameters_iterator: context.parameters.iter(),
        }
    }

    pub fn get(&self, index: usize) -> Option<&'a Variant> {
        self.context
            .parameters
            .get_at(index)
            .map(|a| self.context.evaluate_parameter(a))
    }
}

impl<'a> Iterator for ParameterValues<'a> {
    type Item = &'a Variant;

    fn next(&mut self) -> Option<Self::Item> {
        match self.parameters_iterator.next() {
            Some(arg) => {
                let value = self.context.evaluate_parameter(arg);
                Some(value)
            }
            None => None,
        }
    }
}
