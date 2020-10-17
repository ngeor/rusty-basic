use crate::interpreter::context::Context;
use crate::interpreter::input::Input;
use crate::interpreter::io::FileManager;
use crate::interpreter::printer::Printer;
use crate::interpreter::stdlib::Stdlib;
use crate::linter::UserDefinedTypes;

pub trait InterpreterTrait {
    type TStdlib: Stdlib;
    type TStdIn: Input;
    type TStdOut: Printer;
    type TLpt1: Printer;

    fn context(&self) -> &Context;

    fn context_mut(&mut self) -> &mut Context;

    fn file_manager(&mut self) -> &mut FileManager;

    fn stdlib(&self) -> &Self::TStdlib;

    fn stdlib_mut(&mut self) -> &mut Self::TStdlib;

    fn user_defined_types(&self) -> &UserDefinedTypes;

    fn stdin(&mut self) -> &mut Self::TStdIn;

    fn stdout(&mut self) -> &mut Self::TStdOut;

    fn lpt1(&mut self) -> &mut Self::TLpt1;
}
