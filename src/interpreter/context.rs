use crate::interpreter::arguments::Arguments;
use crate::interpreter::variables::Variables;
use crate::parser::{DimName, Name};
use crate::variant::Variant;

pub struct Context {
    states: Vec<State>,
    memory_blocks: Vec<Variables>,
}

impl Context {
    pub fn new() -> Self {
        let global_variables = Variables::new();
        let memory_blocks = vec![global_variables];
        let root_state = State::new_normal(0);
        let states = vec![root_state];
        Self {
            states,
            memory_blocks,
        }
    }

    fn current_memory_block_index(&self) -> usize {
        self.state().idx
    }

    pub fn begin_collecting_arguments(&mut self) {
        let current_memory_block_index: usize = self.current_memory_block_index();
        // build argument state that shares memory with the last context
        self.states
            .push(State::new_arguments(current_memory_block_index));
    }

    pub fn stop_collecting_arguments(&mut self) {
        // current state must be argument collecting state
        let last_context = self.states.pop().expect("Empty states!");
        let arguments = last_context.arguments.expect("Expected argument state");
        let variables = Self::arguments_to_variables(arguments);
        // push state as last on the list
        let next_memory_block_index = self.memory_blocks.len();
        let new_state = State::new_normal(next_memory_block_index);
        self.memory_blocks.push(variables);
        self.states.push(new_state);
    }

    fn arguments_to_variables(arguments: Arguments) -> Variables {
        let mut variables: Variables = Variables::new();
        for (opt_param, arg) in arguments.into_iter() {
            match opt_param {
                Some(param_name) => variables.insert_param(param_name, arg),
                None => variables.insert_unnamed(arg),
            }
        }
        variables
    }

    pub fn pop(&mut self) {
        let state = self.states.pop().expect("Empty states!");
        if state.arguments.is_some() {
            // must be NormalState (if array indices, use the drop array method)
            panic!("Expected normal state");
        }
        // TODO use some reference counting here
        // was it pointing to the last? but do not drop global module state
        if state.idx + 1 == self.memory_blocks.len() && self.memory_blocks.len() > 1 {
            self.memory_blocks.pop().expect("Empty variables");
        }
    }

    pub fn push_error_handler_context(&mut self) {
        // TODO drop all ArgumentState until we hit the first NormalState
        // build new NormalState referencing the global memory
        let new_state = State::new_normal(0);
        // push as last
        self.states.push(new_state);
    }

    fn state(&self) -> &State {
        self.states.last().expect("Empty states!")
    }

    fn state_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("Empty states!")
    }

    // needed due to DIM SHARED
    pub fn global_state_mut(&mut self) -> &mut Variables {
        self.memory_blocks.first_mut().unwrap()
    }

    fn get_variables(&self) -> &Variables {
        self.memory_blocks
            .get(self.current_memory_block_index())
            .expect("internal error")
    }

    pub fn get_variables_mut(&mut self) -> &mut Variables {
        let current_memory_block_index = self.current_memory_block_index();
        self.memory_blocks
            .get_mut(current_memory_block_index)
            .expect("internal error")
    }

    pub fn get_arguments_mut(&mut self) -> &mut Arguments {
        self.state_mut()
            .arguments
            .as_mut()
            .expect("Not collecting arguments!")
    }

    pub fn drop_arguments_for_array_allocation(&mut self) -> Arguments {
        self.states.pop().unwrap().arguments.unwrap()
    }

    // TODO the remaining functions are from Variables, maybe not expose them or move them into a trait if needed

    pub fn get(&self, idx: usize) -> Option<&Variant> {
        self.get_variables().get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> Option<&mut Variant> {
        self.get_variables_mut().get_mut(idx)
    }

    pub fn set_variable(&mut self, dim_name: DimName, value: Variant) {
        self.get_variables_mut().insert_dim(dim_name, value);
    }

    pub fn variables_len(&self) -> usize {
        self.get_variables().len()
    }

    pub fn get_or_create(&mut self, var_name: Name) -> &mut Variant {
        self.get_variables_mut().get_or_create(var_name)
    }

    #[cfg(test)]
    pub fn get_by_name(&self, name: &Name) -> Variant {
        self.get_variables()
            .get_by_name(name)
            .map(Clone::clone)
            .expect("Variable not found")
    }
}

struct State {
    idx: usize,
    arguments: Option<Arguments>,
}

impl State {
    pub fn new_normal(idx: usize) -> Self {
        Self {
            idx,
            arguments: None,
        }
    }

    pub fn new_arguments(idx: usize) -> Self {
        Self {
            idx,
            arguments: Some(Arguments::new()),
        }
    }
}
