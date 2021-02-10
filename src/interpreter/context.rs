use crate::interpreter::arguments::Arguments;
use crate::interpreter::arguments_stack::ArgumentsStack;
use crate::interpreter::variables::Variables;
use crate::parser::{
    BareName, DimName, DimType, ExpressionType, HasExpressionType, Name, TypeQualifier,
    UserDefinedTypes,
};
use crate::variant::Variant;
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
            variables,
            parameter_count,
            arguments_stack: ArgumentsStack::new(),
            parent: Some(Box::new(self)),
        }
    }

    pub fn set_variable(&mut self, dim_name: DimName, value: Variant) {
        let DimName {
            bare_name,
            dim_type,
            ..
        } = dim_name;
        match dim_type {
            DimType::BuiltIn(qualifier, _) => {
                self.set_variable_built_in(bare_name, qualifier, value);
            }
            DimType::FixedLengthString(_, _) => {
                self.set_variable_built_in(bare_name, TypeQualifier::DollarString, value);
            }
            DimType::UserDefined(_) => {
                self.set_variable_user_defined(bare_name, value);
            }
            DimType::Array(_, box_element_type) => {
                let element_type = box_element_type.expression_type();
                match element_type {
                    ExpressionType::BuiltIn(q) => {
                        self.set_variable_built_in(bare_name, q, value);
                    }
                    ExpressionType::FixedLengthString(_) => {
                        self.set_variable_built_in(bare_name, TypeQualifier::DollarString, value);
                    }
                    _ => self.set_variable_user_defined(bare_name, value),
                }
            }
            DimType::Bare => panic!("Unresolved type"),
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

    // ========================================================
    // used to be ArgsContext
    // ========================================================

    pub fn arguments_stack(&mut self) -> &mut ArgumentsStack {
        &mut self.arguments_stack
    }

    pub fn get_r_value_by_name(&self, name: &Name) -> Option<&Variant> {
        let bare_name: &BareName = name.bare_name();
        match name.qualifier() {
            Some(qualifier) => self.variables.get_built_in(bare_name, qualifier),
            None => {
                // can only be user defined type or array of user defined types
                self.variables.get_user_defined(bare_name)
            }
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

    /// Gets or creates a variable by the given name.
    /// If the variable does not exist, it is initialized with 0.
    pub fn get_or_create(&mut self, var_name: Name) -> &mut Variant {
        self.variables.get_or_create(var_name)
    }

    /// Gets the global (module) context.
    pub fn global_context(&mut self) -> &mut Self {
        if self.parent.is_none() {
            self
        } else {
            self.parent.as_mut().unwrap().global_context()
        }
    }

    /// Gets or creates a variable by the given name in the global (module) context.
    /// If the variables does not exist, it is initialized with 0.
    pub fn get_or_create_global(&mut self, var_name: Name) -> &mut Variant {
        self.global_context().get_or_create(var_name)
    }
}
