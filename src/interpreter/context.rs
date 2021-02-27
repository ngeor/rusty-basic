use crate::interpreter::arguments::Arguments;
use crate::interpreter::arguments_stack::ArgumentsStack;
use crate::interpreter::variables::Variables;
use crate::parser::{
    BareName, DimName, DimType, ExpressionType, HasExpressionType, Name, TypeQualifier,
};
use crate::variant::Variant;

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

// TODO state machine to understand context better, perhaps with the state pattern as in the rust docs
// TODO maybe instead of Variables have some sort of VariableReference class pointing to a VariableManager with an index

// Context
//      |--> Argument Context (can use variables of parent context in read-only mode)
//                              has a vec of future arguments for the sub call
//               |--^^ (recursively stacked for nested sub calls)
//           |--> Child Context (has its own variables, initialized by the argument context stack)

#[derive(Debug)]
pub struct Context {
    variables: Variables,

    /// Preparing arguments for the next call
    arguments_stack: ArgumentsStack,
}

impl Context {
    pub fn new() -> Self {
        Self {
            variables: Variables::new(),
            arguments_stack: ArgumentsStack::new(),
        }
    }

    pub fn push(&mut self) -> Self {
        let arguments: Arguments = self.arguments_stack.pop();
        let mut variables = Variables::new();
        for (opt_param, arg) in arguments.into_iter() {
            match opt_param {
                Some(param_name) => variables.insert_param(param_name, arg),
                None => variables.insert_unnamed(arg),
            }
        }
        Self {
            variables,
            arguments_stack: ArgumentsStack::new(),
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

    pub fn arguments_stack(&mut self) -> &mut ArgumentsStack {
        &mut self.arguments_stack
    }

    #[deprecated]
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

    /// Gets the number of variables in the context.
    /// For built-in subs / functions, this is equivalent to the number of
    /// parameters they were called with.
    pub fn variables_len(&self) -> usize {
        self.variables.len()
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
}

#[derive(Debug)]
pub struct Contexts {
    v: Vec<Option<Context>>,
}

impl Contexts {
    pub fn new() -> Self {
        let context = Context::new();
        Self {
            v: vec![Some(context)],
        }
    }

    pub fn context(&self) -> &Context {
        match self.v.last().unwrap() {
            Some(ctx) => ctx,
            _ => self.global_context(),
        }
    }

    pub fn context_mut(&mut self) -> &mut Context {
        if self.v.last().unwrap().is_some() {
            self.v.last_mut().unwrap().as_mut().unwrap()
        } else {
            self.global_context_mut()
        }
    }

    pub fn push(&mut self) {
        let context = self.context_mut().push();
        self.v.push(Some(context));
    }

    pub fn push_error_handler_context(&mut self) {
        self.v.push(None);
    }

    pub fn pop_error_handler_context(&mut self) {
        self.pop();
    }

    pub fn pop(&mut self) {
        self.v.pop();
    }

    pub fn global_context(&self) -> &Context {
        self.v.first().unwrap().as_ref().unwrap()
    }

    pub fn global_context_mut(&mut self) -> &mut Context {
        self.v.first_mut().unwrap().as_mut().unwrap()
    }
}
