use crate::interpreter::argument::Argument;
use crate::interpreter::arguments::Arguments;
use crate::linter::ParamName;
use std::collections::VecDeque;

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
