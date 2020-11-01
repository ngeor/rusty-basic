use crate::common::StringUtils;
use crate::interpreter::arguments::Arguments;
use crate::interpreter::arguments_stack::ArgumentsStack;
use crate::interpreter::variables::Variables;
use crate::linter::{
    DimName, DimType, ExpressionType, HasExpressionType, Members, UserDefinedTypes,
};
use crate::parser::{BareName, Name, QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::collections::HashMap;
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

#[derive(Debug)]
pub struct Context {
    parent: Option<Box<Context>>,
    user_defined_types: Rc<UserDefinedTypes>,
    constants: HashMap<QualifiedName, Variant>,
    variables: Variables,

    /// Preparing arguments for the next call
    arguments_stack: ArgumentsStack,

    /// The number of parameters of this context.
    parameter_count: usize,
}

impl Context {
    pub fn new(user_defined_types: Rc<UserDefinedTypes>) -> Self {
        Self {
            parent: None,
            user_defined_types,
            constants: HashMap::new(),
            variables: Variables::new(),
            arguments_stack: ArgumentsStack::new(),
            parameter_count: 0, // root context, no parameters
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
        let mut variables = Variables::new();
        for (opt_param, arg) in arguments.into_iter() {
            match opt_param {
                Some(param_name) => variables.insert_param(param_name, arg),
                None => variables.insert_unnamed(arg),
            }
        }
        let parameter_count = variables.len();
        Self {
            user_defined_types: Rc::clone(&self.user_defined_types),
            constants: HashMap::new(),
            variables,
            parameter_count,
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
            DimType::FixedLengthString(_len) => {
                self.set_variable_built_in(bare_name, TypeQualifier::DollarString, value);
            }
            DimType::UserDefined(_) => {
                self.set_variable_user_defined(bare_name, value);
            }
            DimType::Many(_, members) => {
                self.set_variable_member(bare_name, members, value);
            }
            DimType::Array(_, box_element_type) => {
                let element_type = *box_element_type;
                let array_name = DimName::new(bare_name, element_type);
                self.set_variable(array_name, value);
            }
        }
    }

    fn set_variable_built_in(
        &mut self,
        bare_name: BareName,
        qualifier: TypeQualifier,
        value: Variant,
    ) {
        self.variables.insert_built_in(bare_name, qualifier, value);
    }

    fn set_variable_user_defined(&mut self, bare_name: BareName, value: Variant) {
        self.variables.insert_user_defined(bare_name, value);
    }

    fn set_variable_member(&mut self, bare_name: BareName, members: Members, value: Variant) {
        match self.variables.get_user_defined_mut(&bare_name) {
            Some(Variant::VUserDefined(box_user_defined_type_value)) => {
                let name_path = members.name_path();
                box_user_defined_type_value.insert_path(&name_path, value);
            }
            _ => panic!("Expected member variable {} {:?}", bare_name, members),
        }
    }

    // ========================================================
    // used to be ArgsContext
    // ========================================================

    pub fn arguments_stack(&mut self) -> &mut ArgumentsStack {
        &mut self.arguments_stack
    }

    pub fn get_r_value_by_name(&self, name: &Name) -> Option<&Variant> {
        // get a constant or a local thing or a parent constant
        let bare_name: &BareName = name.as_ref();
        match name.qualifier() {
            Some(qualifier) => {
                // constant or variable or global constant
                let qualified_name = QualifiedName::new(bare_name.clone(), qualifier);
                if let Some(v) = self.constants.get(&qualified_name) {
                    Some(v)
                } else if let Some(v) = self.variables.get_built_in(bare_name, qualifier) {
                    Some(v)
                } else {
                    self.get_root_const(&qualified_name)
                }
            }
            None => {
                // can only be user defined type or array of user defined types
                self.variables.get_user_defined(bare_name)
            }
        }
    }

    #[deprecated]
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
                        match self.variables.get_built_in(bare_name, *qualifier) {
                            Some(v) => Some(v),
                            None => {
                                // is it a root constant
                                self.get_root_const(qualified_name)
                            }
                        }
                    }
                }
            }
            DimType::FixedLengthString(_len) => self
                .variables
                .get_built_in(bare_name, TypeQualifier::DollarString),
            DimType::UserDefined(_) => self.variables.get_user_defined(bare_name),
            DimType::Many(_, members) => {
                // is it a variable
                match self.variables.get_user_defined(bare_name) {
                    Some(Variant::VUserDefined(box_user_defined_type_value)) => {
                        let name_path = members.name_path();
                        box_user_defined_type_value.get_path(&name_path)
                    }
                    _ => None,
                }
            }
            DimType::Array(_, _) => todo!(),
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

    pub fn copy_to_parent(&mut self, idx: usize, parent_var_name: &DimName) {
        let v = self.variables.get(idx).expect("Index out of range");

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

    pub fn parameter_count(&self) -> usize {
        self.parameter_count
    }

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.variables.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.variables.get_mut(idx)
    }

    pub fn get_or_create(&mut self, var_name: Name) -> &mut Variant {
        self.variables.get_or_create(var_name)
    }
}
