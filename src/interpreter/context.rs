use crate::common::{CaseInsensitiveString, StringUtils};
use crate::interpreter::arguments::Arguments;
use crate::interpreter::arguments_stack::ArgumentsStack;
use crate::linter::{
    DimName, DimType, ExpressionType, HasExpressionType, Members, ParamName, ParamType,
    UserDefinedTypes,
};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::rc::Rc;

// TODO fix all unimplemented

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

#[derive(Debug)]
pub struct Context {
    parent: Option<Box<Context>>,
    user_defined_types: Rc<UserDefinedTypes>,
    constants: HashMap<QualifiedName, Variant>,
    variables: HashMap<QualifiedName, Variant>,
    user_defined_type_variables: HashMap<CaseInsensitiveString, Variant>,

    /// Maps the order of the parameter to the name
    unnamed: HashMap<u8, Name>,

    /// Preparing arguments for the next call
    arguments_stack: ArgumentsStack,
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
            unnamed: HashMap::new(),
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
        let mut variables: HashMap<QualifiedName, Variant> = HashMap::new();
        let mut user_defined_type_variables: HashMap<BareName, Variant> = HashMap::new();
        let mut unnamed: HashMap<u8, Name> = HashMap::new();

        let mut idx: u8 = 0;
        match arguments {
            Arguments::Unnamed(v) => {
                for arg in v {
                    let bare_name: CaseInsensitiveString = format!("{}", idx).into();
                    match TypeQualifier::try_from(&arg) {
                        Ok(q) => {
                            unnamed.insert(idx, Name::new(bare_name.clone(), Some(q)));
                            variables.insert(QualifiedName::new(bare_name, q), arg);
                        }
                        Err(_) => {
                            unnamed.insert(idx, Name::new(bare_name.clone(), None));
                            user_defined_type_variables.insert(bare_name, arg);
                        }
                    }

                    idx += 1;
                }
            }
            Arguments::Named(map) => {
                for (param_name, arg) in map {
                    let (bare_name, param_type) = param_name.into_inner();
                    match param_type {
                        ParamType::BuiltIn(q) => {
                            unnamed.insert(idx, Name::new(bare_name.clone(), Some(q)));
                            variables.insert(QualifiedName::new(bare_name, q), arg);
                        }
                        ParamType::UserDefined(_) => {
                            unnamed.insert(idx, Name::new(bare_name.clone(), None));
                            user_defined_type_variables.insert(bare_name, arg);
                        }
                    }

                    idx += 1;
                }
            }
        }
        Self {
            user_defined_types: Rc::clone(&self.user_defined_types),
            constants: HashMap::new(),
            variables,
            user_defined_type_variables,
            unnamed,
            arguments_stack: ArgumentsStack::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn set_constant(&mut self, qualified_name: QualifiedName, value: Variant) {
        self.constants.insert(qualified_name, value);
    }

    pub fn set_variable(&mut self, dim_name: DimName, value: Variant) {
        let (bare_name, dim_type) = dim_name.into_inner();
        match dim_type {
            DimType::BuiltIn(qualifier) => {
                self.set_variable_built_in(bare_name, qualifier, value);
            }
            // todo rollback this
            DimType::FixedLengthString(_len) => {
                self.set_variable_built_in(bare_name, TypeQualifier::DollarString, value);
            }
            DimType::UserDefined(_) => {
                self.set_variable_user_defined(bare_name, value);
            }
            DimType::Many(_, members) => {
                self.set_variable_member(bare_name, members, value);
            }
        }
    }

    fn set_variable_built_in(
        &mut self,
        bare_name: BareName,
        qualifier: TypeQualifier,
        value: Variant,
    ) {
        self.variables
            .insert(QualifiedName::new(bare_name, qualifier), value);
    }

    fn set_variable_user_defined(&mut self, bare_name: BareName, value: Variant) {
        self.user_defined_type_variables.insert(bare_name, value);
    }

    fn set_variable_member(&mut self, bare_name: BareName, members: Members, value: Variant) {
        match self.user_defined_type_variables.get_mut(&bare_name) {
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

    // ========================================================
    // used to be ArgsContext
    // ========================================================

    pub fn arguments_stack(&mut self) -> &mut ArgumentsStack {
        &mut self.arguments_stack
    }

    pub fn get_r_value(&self, name: &DimName) -> Option<&Variant> {
        // get a constant or a local thing or a parent constant
        let bare_name: &BareName = name.as_ref();
        match name.dim_type() {
            DimType::BuiltIn(qualifier) => {
                // is it a constant
                let qualified_name = &QualifiedName::new(bare_name.clone(), *qualifier);
                match self.constants.get(qualified_name) {
                    Some(v) => Some(v),
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
            DimType::FixedLengthString(_len) => {
                let qualified_name =
                    QualifiedName::new(bare_name.clone(), TypeQualifier::DollarString);
                self.variables.get(&qualified_name)
            }
            DimType::UserDefined(_) => {
                // is it a variable
                match self.user_defined_type_variables.get(bare_name) {
                    Some(v) => Some(v),
                    None => None,
                }
            }
            DimType::Many(_, members) => {
                // is it a variable
                match self.user_defined_type_variables.get(bare_name) {
                    Some(Variant::VUserDefined(box_user_defined_type_value)) => {
                        let name_path = members.name_path();
                        box_user_defined_type_value.get_path(&name_path)
                    }
                    _ => None,
                }
            }
        }
    }

    // ========================================================
    // private
    // ========================================================

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

    pub fn copy_to_parent(&mut self, param_name: &ParamName, parent_var_name: &DimName) {
        let bare_param_name: &BareName = param_name.as_ref();
        let v = match param_name.param_type() {
            ParamType::BuiltIn(q) => self
                .variables
                .get(&QualifiedName::new(bare_param_name.clone(), *q))
                .expect(&format!("should have built-in variable {:?}", param_name)),
            ParamType::UserDefined(_) => self
                .user_defined_type_variables
                .get(bare_param_name)
                .expect("should have user defined variable"),
        };

        // if the parent_var_name is fixed length string, trim the value
        let v = match parent_var_name.dim_type().expression_type() {
            ExpressionType::FixedLengthString(len) => match v {
                Variant::VString(s) => Variant::VString(s.clone().fix_length(len as usize)),
                _ => v.clone(),
            },
            _ => v.clone(),
        };

        match &mut self.parent {
            Some(p) => {
                p.set_variable(parent_var_name.clone(), v);
            }
            None => panic!("Stack underflow in copy_to_parent"),
        }
    }

    // TODO move the rest into a special structure 'unnamed' or so
    pub fn get(&self, idx: u8) -> Option<&Variant> {
        match self.unnamed.get(&idx) {
            Some(Name::Bare(b)) => self.user_defined_type_variables.get(b),
            Some(Name::Qualified(q)) => self.variables.get(q),
            None => None,
        }
    }

    pub fn parameter_count(&self) -> u8 {
        self.unnamed.len() as u8
    }

    pub fn set(&mut self, idx: u8, value: Variant) {
        match self.unnamed.get(&idx) {
            Some(Name::Bare(b)) => {
                self.user_defined_type_variables.insert(b.clone(), value);
            }
            Some(Name::Qualified(q)) => {
                self.variables.insert(q.clone(), value);
            }
            None => panic!("index out of range"),
        }
    }
}
