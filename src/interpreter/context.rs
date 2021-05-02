use crate::built_ins::BuiltInFunction;
use crate::common::QError;
use crate::instruction_generator::{Path, RootPath};
use crate::interpreter::arguments::Arguments;
use crate::interpreter::variables::Variables;
use crate::linter::SubprogramName;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::{AsciiSize, QBNumberCast, Variant};
use std::collections::HashMap;

// This is an arbitrary value, not what QBasic is doing
pub const VAR_SEG_BASE: usize = 4_096;

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

    pub fn calculate_varptr(&self, path: &Path) -> Result<usize, QError> {
        match path {
            Path::Root(RootPath { name, shared }) => {
                // figure out the memory block where the variable lives
                let memory_block_index = if *shared {
                    0
                } else {
                    self.caller_variables_memory_block_index()
                };
                let mut result: usize = 0;
                for i in 0..memory_block_index {
                    // add the total bytes of the previous variables in previous memory blocks
                    result += self.memory_blocks[i].variables.ascii_size();
                }
                // add the varptr of this variable
                result += self.memory_blocks[memory_block_index]
                    .variables
                    .calculate_var_ptr(name);
                Ok(result)
            }
            Path::ArrayElement(parent_path, indices) => {
                // array elements are relative to the array, so no need to add previous items
                match self.find_value_in_caller_context(parent_path.as_ref())? {
                    Variant::VArray(v_arr) => {
                        let int_indices: Vec<i32> = indices.try_cast()?;
                        v_arr.address_offset_of_element(&int_indices)
                    }
                    _ => panic!("Expected array"),
                }
            }
            Path::Property(parent_path, property_name) => {
                let parent_var_ptr = self.calculate_varptr(parent_path.as_ref())?;
                match self.find_value_in_caller_context(parent_path)? {
                    Variant::VUserDefined(v_u) => {
                        Ok(v_u.address_offset_of_property(property_name) + parent_var_ptr)
                    }
                    _ => panic!("Expected user defined type"),
                }
            }
        }
    }

    fn find_value_in_caller_context(&self, path: &Path) -> Result<&Variant, QError> {
        match path {
            Path::Root(RootPath { name, shared }) => {
                let memory_block_index = if *shared {
                    0
                } else {
                    self.caller_variables_memory_block_index()
                };
                self.memory_blocks[memory_block_index]
                    .variables
                    .get_by_name(name)
                    .ok_or(QError::VariableRequired)
            }
            Path::ArrayElement(parent_path, indices) => {
                match self.find_value_in_caller_context(parent_path.as_ref())? {
                    Variant::VArray(v_arr) => {
                        let int_indices: Vec<i32> = indices.try_cast()?;
                        v_arr.get_element(&int_indices)
                    }
                    _ => panic!("Expected array"),
                }
            }
            Path::Property(parent_path, property_name) => {
                match self.find_value_in_caller_context(parent_path.as_ref())? {
                    Variant::VUserDefined(v_u) => {
                        v_u.get(property_name).ok_or(QError::ElementNotDefined)
                    }
                    _ => panic!("Expected user defined type"),
                }
            }
        }
    }

    pub fn calculate_varseg(&self, path: &Path) -> usize {
        match path {
            Path::Root(RootPath { .. }) => {
                // an ordinary variable (even an Array, but not an Array Element)
                // they share the same data segment
                VAR_SEG_BASE
            }
            Path::ArrayElement(parent_path, ..) => {
                if let Path::Root(RootPath { name, shared }) = parent_path.as_ref() {
                    // figure out the memory block index where the Array is defined
                    let memory_block_index = if *shared {
                        0
                    } else {
                        self.caller_variables_memory_block_index()
                    };
                    let mut result: usize = 0;
                    // add one segment for every array defined in parent memory blocks
                    for i in 0..memory_block_index {
                        result += self.memory_blocks[i].variables.array_names().count();
                    }
                    // add one segment for every array defined in the memory block of the array, until we find the array name
                    result += self.memory_blocks[memory_block_index]
                        .variables
                        .array_names()
                        .take_while(|p| *p != name)
                        .count();
                    // add the array itself
                    result += 1;
                    // add the base segment
                    result += VAR_SEG_BASE;
                    result
                } else {
                    panic!("Expected Path::Root to be parent of Path::ArrayElement");
                }
            }
            Path::Property(parent_path, ..) => self.calculate_varseg(parent_path.as_ref()),
        }
    }

    pub fn peek(&self, seg: usize, address: usize) -> Result<u8, QError> {
        if seg < VAR_SEG_BASE || seg >= 65_536 {
            return Err(QError::SubscriptOutOfRange);
        }
        if seg == VAR_SEG_BASE {
            // not an array element
            let mut offset: usize = 0;
            for block in self.memory_blocks.iter() {
                for var in block.variables.iter() {
                    let len = var.ascii_size();
                    if offset <= address && address < offset + len {
                        return var.peek_non_array(address - offset);
                    }

                    offset += len;
                }
            }
        } else {
            let mut idx = 1;
            for block in self.memory_blocks.iter() {
                for var in block.variables.iter() {
                    if let Variant::VArray(v_arr) = var {
                        if idx == seg - VAR_SEG_BASE {
                            // found the array
                            return v_arr.peek_array_element(address);
                        } else {
                            idx += 1;
                        }
                    }
                }
            }
        }
        Err(QError::SubscriptOutOfRange)
    }

    pub fn poke(&mut self, seg: usize, address: usize, byte_value: u8) -> Result<(), QError> {
        if seg < VAR_SEG_BASE || seg >= 65_536 {
            return Err(QError::SubscriptOutOfRange);
        }
        if seg == VAR_SEG_BASE {
            // not an array element
            let mut offset: usize = 0;
            for block in self.memory_blocks.iter_mut() {
                for var in block.variables.iter_mut() {
                    let len = var.ascii_size();
                    if offset <= address && address < offset + len {
                        return var.poke_non_array(address - offset, byte_value);
                    }

                    offset += len;
                }
            }
        } else {
            let mut idx = 1;
            for block in self.memory_blocks.iter_mut() {
                for var in block.variables.iter_mut() {
                    if let Variant::VArray(v_arr) = var {
                        if idx == seg - VAR_SEG_BASE {
                            // found the array
                            return v_arr.poke_array_element(address, byte_value);
                        } else {
                            idx += 1;
                        }
                    }
                }
            }
        }
        Err(QError::SubscriptOutOfRange)
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
