use crate::interpreter::context::Context;
use crate::interpreter::input::Input;
use crate::interpreter::io::FileManager;
use crate::interpreter::printer::Printer;
use crate::interpreter::registers::Registers;
use crate::interpreter::stdlib::Stdlib;
use crate::linter::UserDefinedTypes;

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
}
