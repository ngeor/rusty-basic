use crate::instruction_generator::Path;
use crate::interpreter::context::Context;
use crate::interpreter::data_segment::DataSegment;
use crate::interpreter::io::{FileManager, Input, Printer};
use crate::interpreter::registers::{RegisterStack, Registers};
use crate::interpreter::Stdlib;
use crate::parser::UserDefinedTypes;
use crate::variant::Variant;
use std::collections::VecDeque;

pub trait InterpreterTrait {
    type TStdlib: Stdlib;
    type TStdIn: Input;
    type TStdOut: Printer;
    type TLpt1: Printer;

    /// Offers system calls
    fn stdlib(&self) -> &Self::TStdlib;

    /// Offers system calls
    fn stdlib_mut(&mut self) -> &mut Self::TStdlib;

    /// Offers file I/O
    fn file_manager(&mut self) -> &mut FileManager;

    /// Abstracts the standard input
    fn stdin(&mut self) -> &mut Self::TStdIn;

    /// Abstracts the standard output
    fn stdout(&mut self) -> &mut Self::TStdOut;

    /// Abstracts the LPT1 printer
    fn lpt1(&mut self) -> &mut Self::TLpt1;

    /// Holds the definition of user defined types
    fn user_defined_types(&self) -> &UserDefinedTypes;

    /// Contains variables and constants, collects function/sub arguments.
    fn context(&self) -> &Context;

    /// Contains variables and constants, collects function/sub arguments.
    fn context_mut(&mut self) -> &mut Context;

    /// Holds the "registers" of the CPU
    fn registers(&self) -> &Registers;

    /// Holds the "registers" of the CPU
    fn registers_mut(&mut self) -> &mut Registers;

    fn register_stack(&mut self) -> &mut RegisterStack;

    fn by_ref_stack(&mut self) -> &mut VecDeque<Variant>;

    fn take_function_result(&mut self) -> Option<Variant>;

    fn set_function_result(&mut self, v: Variant);

    // TODO can this VecDeque be replaced by a simple Option?
    fn var_path_stack(&mut self) -> &mut VecDeque<Path>;

    fn data_segment(&mut self) -> &mut DataSegment;
}
