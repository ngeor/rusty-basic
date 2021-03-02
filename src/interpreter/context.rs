use crate::built_ins::BuiltInFunction;
use crate::interpreter::arguments::Arguments;
use crate::interpreter::variables::Variables;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::Variant;
use std::collections::HashMap;

pub struct Context {
    states: Vec<State>,
    memory_blocks: Vec<Variables>,
    // memory block index -> count
    memory_block_ref_count: HashMap<usize, usize>,
}

impl Context {
    pub fn new() -> Self {
        let global_variables = Variables::new();
        let memory_blocks = vec![global_variables];
        let root_state = State::new(0, false);
        let states = vec![root_state];
        let mut memory_block_ref_count: HashMap<usize, usize> = HashMap::new();
        memory_block_ref_count.insert(0, 1);
        Self {
            states,
            memory_blocks,
            memory_block_ref_count,
        }
    }

    pub fn begin_collecting_arguments(&mut self) {
        // build argument state that shares memory with the last context
        let current_memory_block_index: usize = self.current_memory_block_index();
        self.do_push_existing(current_memory_block_index, true);
    }

    pub fn stop_collecting_arguments(&mut self) {
        // current state must be argument collecting state
        let arguments = self.do_pop().arguments.expect("Expected argument state");
        let variables = Variables::from(arguments);
        self.do_push_new(variables);
    }

    pub fn pop(&mut self) {
        let state = self.do_pop();
        if state.arguments.is_some() {
            // must be NormalState (if array indices, use the drop array method)
            panic!("Expected normal state");
        }
    }

    pub fn push_error_handler_context(&mut self) {
        // drop all ArgumentState until we hit the first NormalState
        while self.states.last().unwrap().arguments.is_some() {
            self.do_pop();
        }
        self.do_push_existing(0, false);
    }

    // needed due to DIM SHARED
    pub fn global_variables_mut(&mut self) -> &mut Variables {
        self.memory_blocks.first_mut().unwrap()
    }

    pub fn variables(&self) -> &Variables {
        self.memory_blocks
            .get(self.current_memory_block_index())
            .expect("internal error")
    }

    pub fn variables_mut(&mut self) -> &mut Variables {
        let current_memory_block_index = self.current_memory_block_index();
        self.memory_blocks
            .get_mut(current_memory_block_index)
            .expect("internal error")
    }

    pub fn arguments_mut(&mut self) -> &mut Arguments {
        self.state_mut()
            .arguments
            .as_mut()
            .expect("Not collecting arguments!")
    }

    pub fn drop_arguments_for_array_allocation(&mut self) -> Arguments {
        self.do_pop()
            .arguments
            .expect("Expected state with arguments")
    }

    pub fn set_built_in_function_result<V>(&mut self, built_in_function: BuiltInFunction, value: V)
    where
        Variant: From<V>,
    {
        let q: TypeQualifier = TypeQualifier::from(&built_in_function);
        let bare_name: BareName = BareName::from(built_in_function);
        self.variables_mut()
            .insert_built_in(bare_name, q, Variant::from(value));
    }

    #[cfg(test)]
    pub fn get_by_name(&self, name: &crate::parser::Name) -> Variant {
        self.variables()
            .get_by_name(name)
            .map(Clone::clone)
            .expect("Variable not found")
    }

    fn state(&self) -> &State {
        self.states.last().expect("Empty states!")
    }

    fn state_mut(&mut self) -> &mut State {
        self.states.last_mut().expect("Empty states!")
    }

    fn current_memory_block_index(&self) -> usize {
        self.state().memory_block_index
    }

    fn do_push_new(&mut self, variables: Variables) {
        let next_memory_block_index = self.memory_blocks.len();
        self.memory_blocks.push(variables);
        self.do_push_existing(next_memory_block_index, false);
    }

    fn do_push_existing(&mut self, memory_block_index: usize, arguments: bool) {
        let state = State::new(memory_block_index, arguments);
        self.states.push(state);
        self.increase_ref_count(memory_block_index);
    }

    fn do_pop(&mut self) -> State {
        let state = self.states.pop().expect("States underflow");
        let removed_from_rc = self.decrease_ref_count(state.memory_block_index);
        if removed_from_rc {
            self.memory_blocks.remove(state.memory_block_index);
        }
        state
    }

    fn increase_ref_count(&mut self, memory_block_index: usize) {
        let existing_count = match self.memory_block_ref_count.get(&memory_block_index) {
            Some(c) => *c,
            _ => 0,
        };
        self.memory_block_ref_count
            .insert(memory_block_index, existing_count + 1);
    }

    fn decrease_ref_count(&mut self, memory_block_index: usize) -> bool {
        let count: usize = *self
            .memory_block_ref_count
            .get(&memory_block_index)
            .expect("Should already have a ref count");
        if count <= 1 {
            self.memory_block_ref_count.remove(&memory_block_index);
            true
        } else {
            self.memory_block_ref_count
                .insert(memory_block_index, count - 1);
            false
        }
    }
}

struct State {
    memory_block_index: usize,
    arguments: Option<Arguments>,
}

impl State {
    pub fn new(memory_block_index: usize, collecting_arguments: bool) -> Self {
        Self {
            memory_block_index,
            arguments: if collecting_arguments {
                Some(Arguments::new())
            } else {
                None
            },
        }
    }
}

impl std::ops::Index<usize> for Context {
    type Output = Variant;

    fn index(&self, index: usize) -> &Variant {
        self.variables()
            .get(index)
            .expect("Variable not found at requested index")
    }
}

impl std::ops::IndexMut<usize> for Context {
    fn index_mut(&mut self, index: usize) -> &mut Variant {
        self.variables_mut()
            .get_mut(index)
            .expect("Variable not found at requested index")
    }
}
