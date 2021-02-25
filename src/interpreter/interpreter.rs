use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionGeneratorResult, Path, PrinterType};
use crate::interpreter::built_ins;
use crate::interpreter::context::*;
use crate::interpreter::default_stdlib::DefaultStdlib;
use crate::interpreter::input::Input;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::FileManager;
use crate::interpreter::lpt1_write::Lpt1Write;
use crate::interpreter::print::PrintInterpreter;
use crate::interpreter::printer::Printer;
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::registers::{RegisterStack, Registers};
use crate::interpreter::stdlib::Stdlib;
use crate::interpreter::write_printer::WritePrinter;
use crate::parser::UserDefinedTypes;
use crate::variant::Variant;
use handlers::{allocation, cast, comparison, logical, math, registers, subprogram, var_path};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::rc::Rc;

pub struct Interpreter<TStdlib: Stdlib, TStdIn: Input, TStdOut: Printer, TLpt1: Printer> {
    /// Offers system calls
    stdlib: TStdlib,

    /// Offers file I/O
    file_manager: FileManager,

    /// Abstracts the standard input
    stdin: TStdIn,

    /// Abstracts the standard output
    stdout: TStdOut,

    /// Abstracts the LPT1 printer
    lpt1: TLpt1,

    /// Holds the definition of user defined types
    user_defined_types: Rc<UserDefinedTypes>,

    /// Contains variables and constants, collects function/sub arguments.
    contexts: Contexts,

    /// Holds the "registers" of the CPU
    register_stack: RegisterStack,

    /// Holds addresses to jump back to
    return_address_stack: Vec<usize>,

    /// Holds addresses to RETURN to after a GOSUB
    go_sub_address_stack: Vec<usize>,

    /// Holds the current call stack
    stacktrace: Vec<Location>,

    /// Holds a path to a variable
    var_path_stack: VecDeque<Path>,

    /// Temporarily holds byref values that are to be copied back to the calling context
    by_ref_stack: VecDeque<Variant>,

    function_result: Option<Variant>,

    value_stack: Vec<Variant>,

    last_error_address: Option<usize>,

    statement_addresses: Vec<usize>,

    print_interpreter: Rc<RefCell<PrintInterpreter>>,
}

impl<TStdlib: Stdlib, TStdIn: Input, TStdOut: Printer, TLpt1: Printer> InterpreterTrait
    for Interpreter<TStdlib, TStdIn, TStdOut, TLpt1>
{
    type TStdlib = TStdlib;
    type TStdIn = TStdIn;
    type TStdOut = TStdOut;
    type TLpt1 = TLpt1;

    fn stdlib(&self) -> &TStdlib {
        &self.stdlib
    }

    fn stdlib_mut(&mut self) -> &mut TStdlib {
        &mut self.stdlib
    }

    fn file_manager(&mut self) -> &mut FileManager {
        &mut self.file_manager
    }

    fn stdin(&mut self) -> &mut Self::TStdIn {
        &mut self.stdin
    }

    fn stdout(&mut self) -> &mut Self::TStdOut {
        &mut self.stdout
    }

    fn lpt1(&mut self) -> &mut Self::TLpt1 {
        &mut self.lpt1
    }

    fn user_defined_types(&self) -> &UserDefinedTypes {
        self.user_defined_types.as_ref()
    }

    fn context(&self) -> &Context {
        &self.contexts.context()
    }

    fn context_mut(&mut self) -> &mut Context {
        self.contexts.context_mut()
    }

    fn global_context_mut(&mut self) -> &mut Context {
        self.contexts.global_context_mut()
    }

    fn registers(&self) -> &Registers {
        self.register_stack.last().unwrap()
    }

    fn registers_mut(&mut self) -> &mut Registers {
        self.register_stack.last_mut().unwrap()
    }

    fn register_stack(&mut self) -> &mut RegisterStack {
        &mut self.register_stack
    }

    fn by_ref_stack(&mut self) -> &mut VecDeque<Variant> {
        &mut self.by_ref_stack
    }

    fn take_function_result(&mut self) -> Option<Variant> {
        self.function_result.take()
    }

    fn set_function_result(&mut self, v: Variant) {
        self.function_result = Some(v);
    }

    fn var_path_stack(&mut self) -> &mut VecDeque<Path> {
        &mut self.var_path_stack
    }
}

pub type DefaultInterpreter = Interpreter<
    DefaultStdlib,
    ReadInputSource<std::io::Stdin>,
    WritePrinter<std::io::Stdout>,
    WritePrinter<Lpt1Write>,
>;

pub fn new_default_interpreter(user_defined_types: UserDefinedTypes) -> DefaultInterpreter {
    let stdlib = DefaultStdlib::new();
    let stdin = ReadInputSource::new(std::io::stdin());
    let stdout = WritePrinter::new(std::io::stdout());
    let lpt1 = WritePrinter::new(Lpt1Write {});
    Interpreter::new(stdlib, stdin, stdout, lpt1, user_defined_types)
}

impl<TStdlib: Stdlib, TStdIn: Input, TStdOut: Printer, TLpt1: Printer>
    Interpreter<TStdlib, TStdIn, TStdOut, TLpt1>
{
    pub fn new(
        stdlib: TStdlib,
        stdin: TStdIn,
        stdout: TStdOut,
        lpt1: TLpt1,
        user_defined_types: UserDefinedTypes,
    ) -> Self {
        let rc_user_defined_types = Rc::new(user_defined_types);
        Interpreter {
            stdlib,
            stdin,
            stdout,
            lpt1,
            contexts: Contexts::new(Rc::clone(&rc_user_defined_types)),
            return_address_stack: vec![],
            go_sub_address_stack: vec![],
            register_stack: vec![Registers::new()],
            stacktrace: vec![],
            file_manager: FileManager::new(),
            user_defined_types: Rc::clone(&rc_user_defined_types),
            var_path_stack: VecDeque::new(),
            by_ref_stack: VecDeque::new(),
            function_result: None,
            value_stack: vec![],
            last_error_address: None,
            statement_addresses: vec![],
            print_interpreter: Rc::new(RefCell::new(PrintInterpreter::new())),
        }
    }

    fn interpret_one(
        &mut self,
        i: usize,
        instruction: &Instruction,
        pos: Location,
        ctx: &mut InterpretOneContext,
    ) -> Result<(), QErrorNode> {
        match instruction {
            Instruction::OnErrorGoTo(address_or_label) => {
                ctx.error_handler = ErrorHandler::Address(address_or_label.address());
            }
            Instruction::OnErrorResumeNext => {
                ctx.error_handler = ErrorHandler::Next;
            }
            Instruction::OnErrorGoToZero => {
                ctx.error_handler = ErrorHandler::None;
            }
            Instruction::PushRegisters => {
                registers::push_registers(self);
            }
            Instruction::PopRegisters => {
                registers::pop_registers(self);
            }
            Instruction::LoadIntoA(v) => {
                registers::load_into_a(self, v);
            }
            Instruction::Cast(q) => {
                cast::cast(self, q).with_err_at(pos)?;
            }
            Instruction::FixLength(l) => {
                cast::fix_length(self, l).with_err_at(pos)?;
            }
            Instruction::CopyAToB => {
                registers::copy_a_to_b(self);
            }
            Instruction::CopyAToC => {
                registers::copy_a_to_c(self);
            }
            Instruction::CopyAToD => {
                registers::copy_a_to_d(self);
            }
            Instruction::CopyCToB => {
                registers::copy_c_to_b(self);
            }
            Instruction::CopyDToA => {
                registers::copy_d_to_a(self);
            }
            Instruction::CopyDToB => {
                registers::copy_d_to_b(self);
            }
            Instruction::Plus => {
                math::plus(self).with_err_at(pos)?;
            }
            Instruction::Minus => {
                math::minus(self).with_err_at(pos)?;
            }
            Instruction::Multiply => {
                math::multiply(self).with_err_at(pos)?;
            }
            Instruction::Divide => {
                math::divide(self).with_err_at(pos)?;
            }
            Instruction::NegateA => {
                logical::negate_a(self).with_err_at(pos)?;
            }
            Instruction::NotA => {
                logical::not_a(self).with_err_at(pos)?;
            }
            Instruction::Equal => {
                comparison::equal(self).with_err_at(pos)?;
            }
            Instruction::NotEqual => {
                comparison::not_equal(self).with_err_at(pos)?;
            }
            Instruction::Less => {
                comparison::less(self).with_err_at(pos)?;
            }
            Instruction::Greater => {
                comparison::greater(self).with_err_at(pos)?;
            }
            Instruction::LessOrEqual => {
                comparison::less_or_equal(self).with_err_at(pos)?;
            }
            Instruction::GreaterOrEqual => {
                comparison::greater_or_equal(self).with_err_at(pos)?;
            }
            Instruction::And => {
                logical::and(self).with_err_at(pos)?;
            }
            Instruction::Or => {
                logical::or(self).with_err_at(pos)?;
            }
            Instruction::JumpIfFalse(address_or_label) => {
                let a = self.registers().get_a();
                let is_true: bool = bool::try_from(a).with_err_at(pos)?;
                if !is_true {
                    ctx.opt_next_index = Some(address_or_label.address());
                }
            }
            Instruction::Jump(address_or_label) => {
                ctx.opt_next_index = Some(address_or_label.address());
            }
            Instruction::BeginCollectArguments => {
                subprogram::begin_collect_arguments(self);
            }
            Instruction::PushStack => {
                self.push_context();
                self.stacktrace.insert(0, pos);
            }
            Instruction::PopStack => {
                self.pop_context();
                self.stacktrace.remove(0);
            }
            Instruction::EnqueueToReturnStack(idx) => {
                subprogram::enqueue_to_return_stack(self, idx);
            }
            Instruction::DequeueFromReturnStack => {
                subprogram::dequeue_from_return_stack(self);
            }
            Instruction::StashFunctionReturnValue(function_name) => {
                subprogram::stash_function_return_value(self, function_name);
            }
            Instruction::UnStashFunctionReturnValue => {
                subprogram::un_stash_function_return_value(self);
            }
            Instruction::PushAToUnnamedArg => {
                subprogram::push_a_to_unnamed_arg(self);
            }
            Instruction::PushNamed(param_name) => {
                subprogram::push_a_to_named_arg(self, param_name);
            }
            Instruction::BuiltInSub(n) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                let run_sub_result = built_ins::run_sub(n, self)?;
                if run_sub_result.halt {
                    ctx.halt = true;
                }
            }
            Instruction::BuiltInFunction(n) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                built_ins::run_function(n, self)?;
            }
            Instruction::Label(_) => (), // no-op
            Instruction::Halt => {
                ctx.halt = true;
            }
            Instruction::PushRet(address) => {
                self.return_address_stack.push(*address);
            }
            Instruction::PopRet => {
                let address = self.return_address_stack.pop().unwrap();
                ctx.opt_next_index = Some(address);
            }
            Instruction::GoSub(address_or_label) => {
                self.go_sub_address_stack.push(i);
                ctx.opt_next_index = Some(address_or_label.address());
            }
            Instruction::Return(opt_address) => match self.go_sub_address_stack.pop() {
                Some(address) => {
                    ctx.opt_next_index = Some(match opt_address {
                        Some(address_or_label) => address_or_label.address(),
                        _ => address + 1,
                    });
                }
                _ => {
                    return Err(QError::ReturnWithoutGoSub).with_err_at(pos);
                }
            },
            Instruction::Resume => match self.last_error_address.take() {
                Some(last_error_address) => {
                    ctx.opt_next_index = Some(self.find_current(last_error_address));
                    self.contexts.pop_error_handler_context();
                }
                _ => {
                    // TODO test this
                    return Err(QError::ResumeWithoutError).with_err_at(pos);
                }
            },
            Instruction::ResumeNext => match self.last_error_address.take() {
                Some(last_error_address) => {
                    ctx.opt_next_index = Some(self.find_next(last_error_address));
                    self.contexts.pop_error_handler_context();
                }
                _ => {
                    // TODO test this
                    return Err(QError::ResumeWithoutError).with_err_at(pos);
                }
            },
            Instruction::ResumeLabel(resume_label) => match self.last_error_address.take() {
                Some(_) => {
                    ctx.opt_next_index = Some(resume_label.address());
                    self.contexts.pop_error_handler_context();
                }
                _ => {
                    // TODO test this
                    return Err(QError::ResumeWithoutError).with_err_at(pos);
                }
            },
            Instruction::Throw(interpreter_error) => {
                return Err(interpreter_error.clone()).with_err_at(pos);
            }
            Instruction::AllocateBuiltIn(q) => {
                allocation::allocate_built_in(self, *q).with_err_at(pos)?;
            }
            Instruction::AllocateFixedLengthString(len) => {
                allocation::allocate_fixed_length_string(self, *len).with_err_at(pos)?;
            }
            Instruction::AllocateArrayIntoA(element_type) => {
                allocation::allocate_array(self, element_type).with_err_at(pos)?;
            }
            Instruction::AllocateUserDefined(user_defined_type_name) => {
                allocation::allocate_user_defined_type(self, user_defined_type_name)
                    .with_err_at(pos)?;
            }
            Instruction::VarPathName(root_path) => {
                var_path::var_path_name(self, root_path.clone());
            }
            Instruction::VarPathIndex => {
                var_path::var_path_index(self);
            }
            Instruction::VarPathProperty(property_name) => {
                var_path::var_path_property(self, property_name);
            }
            Instruction::CopyAToVarPath => {
                var_path::copy_a_to_var_path(self).with_err_at(pos)?;
            }
            Instruction::CopyVarPathToA => {
                var_path::copy_var_path_to_a(self).with_err_at(pos)?;
            }
            Instruction::PushAToValueStack => {
                let v = self.registers().get_a();
                self.value_stack.push(v);
            }
            Instruction::PopValueStackIntoA => {
                let v = self.value_stack.pop().expect("value_stack underflow!");
                self.registers_mut().set_a(v);
            }
            Instruction::PrintSetPrinterType(printer_type) => {
                self.print_interpreter
                    .borrow_mut()
                    .set_printer_type(*printer_type);
            }
            Instruction::PrintSetFileHandle(file_handle) => {
                self.print_interpreter
                    .borrow_mut()
                    .set_file_handle(*file_handle);
            }
            Instruction::PrintSetFormatStringFromA => {
                let encoded_format_string = self.registers().get_a();
                self.print_interpreter.borrow_mut().set_format_string(
                    match encoded_format_string {
                        Variant::VString(s) => Some(s),
                        _ => None,
                    },
                );
            }
            Instruction::PrintComma => {
                let printer = self.choose_printer();
                self.print_interpreter
                    .borrow_mut()
                    .print_comma(printer)
                    .map_err(QError::from)
                    .with_err_at(pos)?;
            }
            Instruction::PrintSemicolon => {
                self.print_interpreter.borrow_mut().print_semicolon();
            }
            Instruction::PrintValueFromA => {
                let v = self.registers().get_a();
                let printer = self.choose_printer();
                self.print_interpreter
                    .borrow_mut()
                    .print_value(printer, v)
                    .with_err_at(pos)?;
            }
            Instruction::PrintEnd => {
                let printer = self.choose_printer();
                self.print_interpreter
                    .borrow_mut()
                    .print_end(printer)
                    .with_err_at(pos)?;
            }
        }
        Ok(())
    }

    fn choose_printer(&self) -> Box<&dyn Printer> {
        let printer_type = self.print_interpreter.borrow().get_printer_type();
        let file_handle = self.print_interpreter.borrow().get_file_handle();
        match printer_type {
            PrinterType::Print => Box::new(&self.stdout),
            PrinterType::LPrint => Box::new(&self.lpt1),
            PrinterType::File => Box::new(
                self.file_manager
                    .try_get_file_info_output(&file_handle)
                    .expect("File not found"),
            ),
        }
    }

    pub fn interpret(
        &mut self,
        instruction_generator_result: InstructionGeneratorResult,
    ) -> Result<(), QErrorNode> {
        let InstructionGeneratorResult {
            instructions,
            statement_addresses,
        } = instruction_generator_result;
        self.statement_addresses = statement_addresses;
        let mut i: usize = 0;
        let mut ctx: InterpretOneContext = InterpretOneContext {
            halt: false,
            error_handler: ErrorHandler::None,
            opt_next_index: None,
        };
        while i < instructions.len() && !ctx.halt {
            let instruction = instructions[i].as_ref();
            let pos = instructions[i].pos();
            match self.interpret_one(i, instruction, pos, &mut ctx) {
                Ok(_) => match ctx.opt_next_index.take() {
                    Some(next_index) => {
                        i = next_index;
                    }
                    _ => {
                        i += 1;
                    }
                },
                Err(e) => {
                    // TODO if was in the middle of building arguments to a sub/function, clean up
                    // TODO what if the error handler is in a different sub / probably linter should catch this
                    // TODO if was calling a sub/function, probably needs to cleanup stack (recursively potentially)
                    match ctx.error_handler {
                        ErrorHandler::Address(handler_address) => {
                            // store error address, so we can call RESUME and RESUME NEXT from within the error handler
                            // TODO not good enough if a sub is called and tries to access a DIM SHARED variable
                            self.contexts.push_error_handler_context();
                            self.last_error_address = Some(i);
                            i = handler_address;
                        }
                        ErrorHandler::Next => {
                            i = self.find_next(i);
                        }
                        ErrorHandler::None => {
                            return Err(e.patch_stacktrace(&self.stacktrace));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn push_context(&mut self) {
        self.contexts.push();
    }

    fn pop_context(&mut self) {
        self.contexts.pop();
    }

    fn find_current(&self, address: usize) -> usize {
        for i in 0..self.statement_addresses.len() {
            let idx = self.statement_addresses[i];
            if idx > address {
                return self.statement_addresses[i - 1];
            }
        }
        *self.statement_addresses.last().unwrap()
    }

    fn find_next(&self, address: usize) -> usize {
        for i in 0..self.statement_addresses.len() {
            let idx = self.statement_addresses[i];
            if idx > address {
                return self.statement_addresses[i];
            }
        }
        usize::MAX
    }
}

/// Context available to the execution of a single instruction.
struct InterpretOneContext {
    /// The instruction handler can set this to `true` in order to terminate
    /// the program (done by the `SYSTEM` and `END` built-ins).
    halt: bool,

    /// The instruction can set a new error handler address (done by
    /// `ON ERROR GOTO` statement).
    error_handler: ErrorHandler,

    /// The instruction can indicate the next address for the control flow.
    /// If not set, control flow will resume to the next statement, if any.
    opt_next_index: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ErrorHandler {
    None,
    Next,
    Address(usize),
}

#[cfg(test)]
mod tests {
    use crate::interpreter::interpreter_trait::InterpreterTrait;
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_interpreter_fixture_hello1() {
        interpret_file("HELLO1.BAS").unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello2() {
        interpret_file("HELLO2.BAS").unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello_s() {
        interpret_file("HELLO_S.BAS").unwrap();
    }

    #[test]
    fn test_interpreter_for_print_10() {
        interpret_file("FOR_PRINT_10.BAS").unwrap();
    }

    #[test]
    fn test_interpreter_for_nested() {
        interpret_file("FOR_NESTED.BAS").unwrap();
    }

    #[test]
    fn test_interpreter_fixture_fib_bas() {
        let mut interpreter = interpret_file_with_raw_input("FIB.BAS", "10").unwrap();
        let output = interpreter.stdout().output_lines();
        assert_eq!(
            output,
            vec![
                "Enter the number of fibonacci to calculate",
                "Fibonacci of   0            is             0",
                "Fibonacci of   1            is             1",
                "Fibonacci of   2            is             1",
                "Fibonacci of   3            is             2",
                "Fibonacci of   4            is             3",
                "Fibonacci of   5            is             5",
                "Fibonacci of   6            is             8",
                "Fibonacci of   7            is             13",
                "Fibonacci of   8            is             21",
                "Fibonacci of   9            is             34",
                "Fibonacci of   10           is             55"
            ]
        );
    }

    #[test]
    fn test_interpreter_fixture_fib_fq_bas() {
        interpret_file_with_raw_input("FIB_FQ.BAS", "11").unwrap();
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let mut interpreter = interpret_file_with_raw_input("INPUT.BAS", "\r\n").unwrap();
        assert_eq!(interpreter.stdout().output_exact(), " 0 \r\n");
    }
}

mod handlers;
