use crate::interpreter::arguments::Arguments;
use crate::parser::ParamName;
use crate::variant::Variant;
use std::collections::VecDeque;

#[derive(Debug)]
pub struct ArgumentsStack {
    stack: VecDeque<Arguments>,
}

impl ArgumentsStack {
    pub fn new() -> Self {
        Self {
            stack: VecDeque::new(),
        }
    }

    pub fn begin_collect_arguments(&mut self) {
        self.stack.push_back(Arguments::new());
    }

    pub fn pop(&mut self) -> Arguments {
        self.stack
            .pop_back()
            .expect("Stack underflow collecting arguments!")
    }

    pub fn push_unnamed(&mut self, arg: Variant) {
        self.demand().push_unnamed(arg);
    }

    pub fn push_named(&mut self, param_name: ParamName, arg: Variant) {
        self.demand().push_named(param_name, arg);
    }

    fn demand(&mut self) -> &mut Arguments {
        self.stack.back_mut().expect("No arguments pushed!")
    }
}
