use crate::instruction_generator::{InstructionGeneratorResult, Path};
use crate::interpreter::context::{Context, VAR_SEG_BASE};
use crate::interpreter::data_segment::DataSegment;
use crate::interpreter::io::{FileManager, Input, Printer};
use crate::interpreter::registers::{RegisterStack, Registers};
use crate::interpreter::screen::Screen;
use crate::interpreter::Stdlib;
use rusty_common::QErrorNode;
use rusty_linter::HasUserDefinedTypes;
use rusty_parser::variant::Variant;
use std::collections::VecDeque;

pub trait InterpreterTrait: HasUserDefinedTypes {
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

    fn screen(&self) -> &dyn Screen;

    fn screen_mut(&mut self) -> &mut dyn Screen;

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

    fn var_path_stack(&mut self) -> &mut VecDeque<Path>;

    ///  Used by the `DATA` statement.
    fn data_segment(&mut self) -> &mut DataSegment;

    fn get_def_seg(&self) -> Option<usize>;

    fn get_def_seg_or_default(&self) -> usize {
        match self.get_def_seg() {
            Some(seg) => seg,
            _ => VAR_SEG_BASE,
        }
    }

    fn set_def_seg(&mut self, def_seg: Option<usize>);

    fn get_last_error_code(&self) -> Option<i32>;

    fn interpret(
        &mut self,
        instruction_generator_result: InstructionGeneratorResult,
    ) -> Result<(), QErrorNode>;
}
