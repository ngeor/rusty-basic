use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionGeneratorResult, Path, PrinterType};
use crate::interpreter::context::*;
use crate::interpreter::data_segment::DataSegment;
use crate::interpreter::default_stdlib::DefaultStdlib;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::{FileManager, Input, Printer};
use crate::interpreter::lpt1_write::Lpt1Write;
use crate::interpreter::print::PrintInterpreter;
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::registers::{RegisterStack, Registers};
use crate::interpreter::screen::{CrossTermScreen, Screen};
use crate::interpreter::write_printer::WritePrinter;
use crate::interpreter::Stdlib;
use crate::parser::UserDefinedTypes;
use crate::variant::{QBNumberCast, Variant};
use handlers::{allocation, cast, comparison, logical, math, registers, subprogram, var_path};
use std::cell::RefCell;
use std::collections::VecDeque;
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

    screen: Box<dyn Screen>,

    /// Holds the definition of user defined types
    user_defined_types: UserDefinedTypes,

    /// Contains variables and constants, collects function/sub arguments.
    context: Context,

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

    last_error_code: Option<i32>,

    print_interpreter: Rc<RefCell<PrintInterpreter>>,

    data_segment: DataSegment,

    def_seg: Option<usize>,
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

    fn screen(&self) -> &dyn Screen {
        self.screen.as_ref()
    }

    fn screen_mut(&mut self) -> &mut dyn Screen {
        self.screen.as_mut()
    }

    fn user_defined_types(&self) -> &UserDefinedTypes {
        &self.user_defined_types
    }

    fn context(&self) -> &Context {
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
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

    fn data_segment(&mut self) -> &mut DataSegment {
        &mut self.data_segment
    }

    fn get_def_seg(&self) -> Option<usize> {
        self.def_seg
    }

    fn set_def_seg(&mut self, def_seg: Option<usize>) {
        self.def_seg = def_seg;
    }

    fn get_last_error_code(&self) -> Option<i32> {
        self.last_error_code
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
    let screen = CrossTermScreen::new();
    Interpreter::new(stdlib, stdin, stdout, lpt1, screen, user_defined_types)
}

impl<TStdlib: Stdlib, TStdIn: Input, TStdOut: Printer, TLpt1: Printer>
    Interpreter<TStdlib, TStdIn, TStdOut, TLpt1>
{
    pub fn new<TScreen: Screen + 'static>(
        stdlib: TStdlib,
        stdin: TStdIn,
        stdout: TStdOut,
        lpt1: TLpt1,
        screen: TScreen,
        user_defined_types: UserDefinedTypes,
    ) -> Self {
        Interpreter {
            stdlib,
            stdin,
            stdout,
            lpt1,
            screen: Box::new(screen),
            context: Context::new(),
            return_address_stack: vec![],
            go_sub_address_stack: vec![],
            register_stack: vec![Registers::new()],
            stacktrace: vec![],
            file_manager: FileManager::new(),
            user_defined_types,
            var_path_stack: VecDeque::new(),
            by_ref_stack: VecDeque::new(),
            function_result: None,
            value_stack: vec![],
            last_error_address: None,
            last_error_code: None,
            print_interpreter: Rc::new(RefCell::new(PrintInterpreter::new())),
            data_segment: DataSegment::new(),
            def_seg: None,
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
            Instruction::Modulo => {
                math::modulo(self).with_err_at(pos)?;
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
                let is_true: bool = a.try_cast().with_err_at(pos)?;
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
                self.context.stop_collecting_arguments();
                self.stacktrace.insert(0, pos);
            }
            Instruction::PushStaticStack(subprogram_name) => {
                self.context
                    .stop_collecting_arguments_static(subprogram_name.clone());
                self.stacktrace.insert(0, pos);
            }
            Instruction::PopStack => {
                self.context.pop();
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
            Instruction::PushUnnamedByVal => {
                subprogram::push_unnamed_arg_by_val(self);
            }
            Instruction::PushUnnamedByRef => {
                subprogram::push_unnamed_arg_by_ref(self);
            }
            Instruction::PushNamed(param_name) => {
                subprogram::push_a_to_named_arg(self, param_name);
            }
            Instruction::BuiltInSub(s) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                crate::built_ins::interpreter::run_sub(s, self).with_err_no_pos()?;
            }
            Instruction::BuiltInFunction(f) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                crate::built_ins::interpreter::run_function(f, self).with_err_no_pos()?;
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
            Instruction::Resume => {
                let last_error_address = self.take_last_error_address().with_err_at(pos)?;
                ctx.opt_next_index = Some(
                    ctx.nearest_statement_finder
                        .find_current(last_error_address),
                );
                self.context.pop();
            }
            Instruction::ResumeNext => {
                let last_error_address = self.take_last_error_address().with_err_at(pos)?;
                ctx.opt_next_index =
                    Some(ctx.nearest_statement_finder.find_next(last_error_address));
                self.context.pop();
            }
            Instruction::ResumeLabel(resume_label) => {
                // not using the last error address but need to clear it which also clears the err code
                self.take_last_error_address().with_err_at(pos)?;
                ctx.opt_next_index = Some(resume_label.address());
                self.context.pop();
            }
            Instruction::Throw(interpreter_error) => {
                return Err(interpreter_error.clone()).with_err_at(pos);
            }
            Instruction::AllocateBuiltIn(q) => {
                allocation::allocate_built_in(self, *q).with_err_at(pos)?;
            }
            Instruction::AllocateFixedLengthString(len) => {
                allocation::allocate_fixed_length_string(self, *len).with_err_at(pos)?;
            }
            // TODO instructions should only accept simple types as arguments
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
                var_path::pop_var_path(self).with_err_at(pos)?;
            }
            Instruction::CopyVarPathToA => {
                var_path::copy_var_path_to_a(self).with_err_at(pos)?;
            }
            Instruction::PopVarPath => {
                var_path::pop_var_path(self).with_err_at(pos)?;
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
            Instruction::IsVariableDefined(dim_name) => {
                debug_assert_ne!(
                    0,
                    self.context.current_memory_block_index(),
                    "Should not be in global scope"
                );
                let variables = self.context.variables();
                let is_variable_defined = variables.get_by_dim_name(&dim_name).is_some();
                self.registers_mut().set_a(is_variable_defined.into());
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
        let mut i: usize = 0;
        let mut ctx: InterpretOneContext = InterpretOneContext {
            halt: false,
            error_handler: ErrorHandler::None,
            opt_next_index: None,
            nearest_statement_finder: NearestStatementFinder::new(statement_addresses),
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
                    self.last_error_code = Some(e.as_ref().get_code());
                    match ctx.error_handler {
                        ErrorHandler::Address(handler_address) => {
                            // store error address, so we can call RESUME and RESUME NEXT from within the error handler
                            self.context.push_error_handler_context();
                            self.last_error_address = Some(i);
                            i = handler_address;
                        }
                        ErrorHandler::Next => {
                            i = ctx.nearest_statement_finder.find_next(i);
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

    /// Gets the instruction address where the most recent error occurred.
    /// Clears that address and also clears the most recent error code.
    fn take_last_error_address(&mut self) -> Result<usize, QError> {
        self.last_error_code = None;
        match self.last_error_address.take() {
            Some(a) => Ok(a),
            None => Err(QError::ResumeWithoutError),
        }
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

    nearest_statement_finder: NearestStatementFinder,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ErrorHandler {
    None,
    Next,
    Address(usize),
}

struct NearestStatementFinder {
    statement_addresses: Vec<usize>,
}

impl NearestStatementFinder {
    pub fn new(statement_addresses: Vec<usize>) -> Self {
        Self {
            statement_addresses,
        }
    }

    pub fn find_current(&self, address: usize) -> usize {
        match self.statement_addresses.binary_search(&address) {
            Ok(_) => address,
            Err(would_be_index) => {
                if would_be_index >= 1 {
                    self.statement_addresses[would_be_index - 1]
                } else {
                    panic!("should never happen")
                }
            }
        }
    }

    pub fn find_next(&self, address: usize) -> usize {
        match self.statement_addresses.binary_search(&address) {
            Ok(existing_index) => {
                if existing_index == self.statement_addresses.len() - 1 {
                    1 + self.statement_addresses[existing_index]
                } else {
                    self.statement_addresses[existing_index + 1]
                }
            }
            Err(would_be_index) => self.statement_addresses[would_be_index],
        }
    }
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
