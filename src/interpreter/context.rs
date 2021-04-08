use crate::built_ins::BuiltInFunction;
use crate::instruction_generator::{Path, RootPath};
use crate::interpreter::arguments::Arguments;
use crate::interpreter::variables::Variables;
use crate::linter::SubprogramName;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::Variant;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Context {
    states: Vec<State>,
    memory_blocks: Vec<MemoryBlock>,
    // static memory blocks (for STATIC function/sub)
    static_memory_blocks: HashMap<SubprogramName, usize>,
}

impl Context {
    pub fn new() -> Self {
        let global_variables = Variables::new();
        let global_memory_block = MemoryBlock::new(global_variables, false);
        let memory_blocks = vec![global_memory_block];
        let root_state = State::new(0, false);
        let states = vec![root_state];
        Self {
            states,
            memory_blocks,
            static_memory_blocks: HashMap::new(),
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
        self.do_push_new(variables, false);
    }

    pub fn stop_collecting_arguments_static(&mut self, subprogram_name: SubprogramName) {
        // current state must be argument collecting state
        let arguments = self.do_pop().arguments.expect("Expected argument state");
        // ensure memory block for this subprogram
        match self.static_memory_blocks.get(&subprogram_name) {
            Some(existing_memory_block_index) => {
                let memory_block_index = *existing_memory_block_index;
                self.memory_blocks[memory_block_index]
                    .variables
                    .apply_arguments(arguments);
                self.do_push_existing(memory_block_index, false);
            }
            _ => {
                let variables = Variables::from(arguments);
                let memory_block_index = self.do_push_new(variables, true);
                self.static_memory_blocks
                    .insert(subprogram_name, memory_block_index);
            }
        }
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

    pub fn global_variables(&self) -> &Variables {
        &self.memory_blocks.first().unwrap().variables
    }

    // needed due to DIM SHARED
    pub fn global_variables_mut(&mut self) -> &mut Variables {
        &mut self.memory_blocks.first_mut().unwrap().variables
    }

    pub fn variables(&self) -> &Variables {
        &self
            .memory_blocks
            .get(self.current_memory_block_index())
            .expect("internal error")
            .variables
    }

    pub fn variables_mut(&mut self) -> &mut Variables {
        let current_memory_block_index = self.current_memory_block_index();
        &mut self
            .memory_blocks
            .get_mut(current_memory_block_index)
            .expect("internal error")
            .variables
    }

    pub fn caller_variables(&self) -> &Variables {
        let memory_block_index = self.caller_variables_memory_block_index();
        &self
            .memory_blocks
            .get(memory_block_index)
            .expect("internal error")
            .variables
    }

    pub fn caller_variables_mut(&mut self) -> &mut Variables {
        let memory_block_index = self.caller_variables_memory_block_index();
        &mut self
            .memory_blocks
            .get_mut(memory_block_index)
            .expect("internal error")
            .variables
    }

    fn caller_variables_memory_block_index(&self) -> usize {
        debug_assert!(self.states.len() >= 2);
        let caller_state = self
            .states
            .get(self.states.len() - 2)
            .expect("Should have caller state");
        caller_state.memory_block_index
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

    pub fn current_memory_block_index(&self) -> usize {
        self.state().memory_block_index
    }

    fn do_push_new(&mut self, variables: Variables, is_static: bool) -> usize {
        let next_memory_block_index = self.memory_blocks.len();
        let memory_block = MemoryBlock::new(variables, is_static);
        self.memory_blocks.push(memory_block);
        self.do_push_state(next_memory_block_index, false);
        next_memory_block_index
    }

    fn do_push_existing(&mut self, memory_block_index: usize, arguments: bool) {
        self.do_push_state(memory_block_index, arguments);
        self.increase_ref_count(memory_block_index);
    }

    fn do_push_state(&mut self, memory_block_index: usize, arguments: bool) {
        let state = State::new(memory_block_index, arguments);
        self.states.push(state);
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
        self.memory_blocks[memory_block_index].increase_ref_count();
    }

    fn decrease_ref_count(&mut self, memory_block_index: usize) -> bool {
        self.memory_blocks[memory_block_index].decrease_ref_count()
    }

    pub fn calculate_varseg(&self, path: &Path) -> usize {
        match path {
            Path::Root(RootPath { shared, .. }) => {
                let index = if *shared {
                    0
                } else {
                    self.caller_variables_memory_block_index()
                };
                let mut result = 0;
                for i in 0..index {
                    result += self.memory_blocks[i].variables.number_of_arrays() + 1;
                }
                result
            }
            Path::ArrayElement(parent_path, ..) => {
                if let Path::Root(RootPath { name, shared }) = parent_path.as_ref() {
                    let memory_block_index = if *shared {
                        0
                    } else {
                        self.caller_variables_memory_block_index()
                    };
                    let grand_parent_varseg = if memory_block_index > 0 {
                        self.memory_blocks[memory_block_index - 1]
                            .variables
                            .number_of_arrays()
                            + 1
                    } else {
                        0
                    };
                    let array_varseg = self.memory_blocks[memory_block_index]
                        .variables
                        .number_of_arrays_until(name);
                    grand_parent_varseg + array_varseg
                } else {
                    panic!("Expected Path::Root to be parent of Path::ArrayElement");
                }
            }
            Path::Property(parent_path, ..) => self.calculate_varseg(parent_path.as_ref()),
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

#[derive(Debug)]
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

/// Represents a memory area local to a sub/function call.
/// The global memory block is assigned to the global module and
/// is also re-used by the global error handler (if one exists).
/// Each function/sub call gets a new memory block which is discarded when
/// the function/sub exits. Static function/subs retain their memory block
/// between calls.
#[derive(Debug)]
pub struct MemoryBlock {
    /// The variables in the memory block.
    variables: Variables,

    /// A reference counter that indicates how many context states are using
    /// this memory block. A memory block is re-used while evaluating the arguments
    /// of a function/sub call and also when the error handler is invoked.
    ref_count: usize,

    /// Determines if this memory block belongs to a static function/sub, which
    /// means it should not be discarded even if the reference counter would
    /// normally indicate so.
    is_static: bool,
}

impl MemoryBlock {
    fn new(variables: Variables, is_static: bool) -> Self {
        Self {
            variables,
            ref_count: 1,
            is_static,
        }
    }

    fn increase_ref_count(&mut self) {
        self.ref_count += 1;
    }

    fn decrease_ref_count(&mut self) -> bool {
        if self.ref_count > 1 {
            // decrease ref count
            self.ref_count -= 1;
            // indicate it cannot be removed
            false
        } else {
            // it can be removed only if it is not static
            !self.is_static
        }
    }
}
