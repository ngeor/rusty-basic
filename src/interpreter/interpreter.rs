use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionNode};
use crate::interpreter::context::*;
use crate::interpreter::default_stdlib::DefaultStdlib;
use crate::interpreter::input::Input;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::io::FileManager;
use crate::interpreter::lpt1_write::Lpt1Write;
use crate::interpreter::printer::Printer;
use crate::interpreter::read_input::ReadInputSource;
use crate::interpreter::registers::{RegisterStack, Registers};
use crate::interpreter::stdlib::Stdlib;
use crate::interpreter::write_printer::WritePrinter;
use crate::interpreter::{built_ins, instruction_handlers};
use crate::linter::UserDefinedTypes;
use crate::parser::{BareName, Name, TypeQualifier};
use crate::variant::{Path, Variant};
use std::cmp::Ordering;
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
    context: Context,

    /// Holds the "registers" of the CPU
    register_stack: RegisterStack,

    /// Holds addresses to jump back to
    return_address_stack: Vec<usize>,

    /// Holds the current call stack
    stacktrace: Vec<Location>,

    /// Holds a path to a variable
    var_path_stack: VecDeque<Path>,

    /// Temporarily holds byref values that are to be copied back to the calling context
    by_ref_stack: VecDeque<Variant>,

    function_result: Option<Variant>,
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
        &self.context
    }

    fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    fn registers(&self) -> &Registers {
        self.register_stack.back().unwrap()
    }

    fn registers_mut(&mut self) -> &mut Registers {
        self.register_stack.back_mut().unwrap()
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
        let mut result = Interpreter {
            stdlib,
            stdin,
            stdout,
            lpt1,
            context: Context::new(Rc::clone(&rc_user_defined_types)),
            return_address_stack: vec![],
            register_stack: VecDeque::new(),
            stacktrace: vec![],
            file_manager: FileManager::new(),
            user_defined_types: Rc::clone(&rc_user_defined_types),
            var_path_stack: VecDeque::new(),
            by_ref_stack: VecDeque::new(),
            function_result: None,
        };
        result.register_stack.push_back(Registers::new());
        result
    }

    fn get_a(&self) -> Variant {
        self.registers().get_a()
    }

    fn get_b(&self) -> Variant {
        self.registers().get_b()
    }

    fn set_a(&mut self, v: Variant) {
        self.registers_mut().set_a(v);
    }

    fn interpret_one(
        &mut self,
        i: &mut usize,
        instruction: &Instruction,
        pos: Location,
        error_handler: &mut Option<usize>,
        exit: &mut bool,
    ) -> Result<(), QErrorNode> {
        match instruction {
            Instruction::SetErrorHandler(idx) => {
                *error_handler = Some(*idx);
            }
            Instruction::PushRegisters => {
                self.register_stack.push_back(Registers::new());
            }
            Instruction::PopRegisters => {
                let old_registers = self.register_stack.pop_back();
                self.set_a(old_registers.unwrap().get_a());
            }
            Instruction::Load(v) => {
                self.set_a(v.clone());
            }
            Instruction::Cast(q) => {
                let v = self.get_a();
                let casted = v.cast(*q).with_err_at(pos)?;
                self.set_a(casted);
            }
            Instruction::FixLength(l) => {
                let v = self.get_a();
                let casted = v.cast(TypeQualifier::DollarString).with_err_at(pos)?;
                self.set_a(match casted {
                    Variant::VString(s) => {
                        let len: usize = *l as usize;
                        Variant::VString(s.fix_length(len))
                    }
                    _ => casted,
                });
            }
            Instruction::CopyAToB => {
                self.registers_mut().copy_a_to_b();
            }
            Instruction::CopyAToC => {
                self.registers_mut().copy_a_to_c();
            }
            Instruction::CopyAToD => {
                self.registers_mut().copy_a_to_d();
            }
            Instruction::CopyCToB => {
                self.registers_mut().copy_c_to_b();
            }
            Instruction::CopyDToA => {
                self.registers_mut().copy_d_to_a();
            }
            Instruction::CopyDToB => {
                self.registers_mut().copy_d_to_b();
            }
            Instruction::SwapAWithB => {
                self.registers_mut().swap_a_with_b();
            }
            Instruction::Plus => {
                instruction_handlers::math::plus(self).with_err_at(pos)?;
            }
            Instruction::Minus => {
                instruction_handlers::math::minus(self).with_err_at(pos)?;
            }
            Instruction::Multiply => {
                instruction_handlers::math::multiply(self).with_err_at(pos)?;
            }
            Instruction::Divide => {
                instruction_handlers::math::divide(self).with_err_at(pos)?;
            }
            Instruction::NegateA => {
                let a = self.get_a();
                let c = a.negate().with_err_at(pos)?;
                self.set_a(c);
            }
            Instruction::NotA => {
                let a = self.get_a();
                let c = a.unary_not().with_err_at(pos)?;
                self.set_a(c);
            }
            Instruction::Equal => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::NotEqual => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order != Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::Less => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order == Ordering::Less;
                self.set_a(is_true.into());
            }
            Instruction::Greater => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order == Ordering::Greater;
                self.set_a(is_true.into());
            }
            Instruction::LessOrEqual => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order == Ordering::Less || order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::GreaterOrEqual => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a.cmp(&b).with_err_at(pos)?;
                let is_true = order == Ordering::Greater || order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::And => {
                instruction_handlers::logical::and(self).with_err_at(pos)?;
            }
            Instruction::Or => {
                instruction_handlers::logical::or(self).with_err_at(pos)?;
            }
            Instruction::JumpIfFalse(resolved_idx) => {
                let a = self.get_a();
                let is_true: bool = bool::try_from(a).with_err_at(pos)?;
                if !is_true {
                    *i = resolved_idx - 1; // the +1 will happen at the end of the loop
                }
            }
            Instruction::Jump(resolved_idx) => {
                *i = resolved_idx - 1;
            }
            Instruction::BeginCollectArguments => {
                self.context.arguments_stack().begin_collect_arguments();
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
                let v = self.context.get(*idx).expect("Should have value").clone();
                self.by_ref_stack.push_back(v);
            }
            Instruction::DequeueFromReturnStack => {
                let v = self
                    .by_ref_stack
                    .pop_front()
                    .expect("by_ref_stack underflow");
                self.registers_mut().set_a(v);
            }
            Instruction::StashFunctionReturnValue(function_name) => {
                let name: Name = Name::Qualified(function_name.clone());
                let v = self.context_mut().get_or_create(name).clone();
                self.function_result = Some(v);
            }
            Instruction::UnStashFunctionReturnValue => {
                let v = self
                    .function_result
                    .take()
                    .expect("Should have function result");
                self.registers_mut().set_a(v);
            }
            Instruction::PushUnnamed => {
                let v = self.get_a();
                self.context.arguments_stack().push_unnamed(v);
            }
            Instruction::PushNamed(param_q_name) => {
                let v = self.get_a();
                self.context
                    .arguments_stack()
                    .push_named(param_q_name.clone(), v);
            }
            Instruction::BuiltInSub(n) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                built_ins::run_sub(n, self)?;
            }
            Instruction::BuiltInFunction(n) => {
                // note: not patching the error pos for built-ins because it's already in-place by Instruction::PushStack
                built_ins::run_function(n, self)?;
            }
            Instruction::UnresolvedJump(_)
            | Instruction::UnresolvedJumpIfFalse(_)
            | Instruction::SetUnresolvedErrorHandler(_) => {
                panic!("Unresolved label {:?} at {:?}", instruction, pos)
            }
            Instruction::Label(_) => (), // no-op
            Instruction::Halt => {
                *exit = true;
            }
            Instruction::PushRet(addr) => {
                self.return_address_stack.push(*addr);
            }
            Instruction::PopRet => {
                let addr = self.return_address_stack.pop().unwrap();
                *i = addr - 1;
            }
            Instruction::Throw(interpreter_error) => {
                return Err(interpreter_error.clone()).with_err_at(pos);
            }
            Instruction::AllocateBuiltIn(q) => {
                instruction_handlers::allocation::allocate_built_in(self, *q).with_err_at(pos)?;
            }
            Instruction::AllocateFixedLengthString(len) => {
                instruction_handlers::allocation::allocate_fixed_length_string(self, *len)
                    .with_err_at(pos)?;
            }
            Instruction::AllocateArray(element_type) => {
                instruction_handlers::allocation::allocate_array(self, element_type)
                    .with_err_at(pos)?;
            }
            Instruction::AllocateUserDefined(user_defined_type_name) => {
                instruction_handlers::allocation::allocate_user_defined_type(
                    self,
                    user_defined_type_name,
                )
                .with_err_at(pos)?;
            }
            Instruction::VarPathName(name) => {
                self.var_path_stack.push_back(Path::Root(name.clone()));
            }
            Instruction::VarPathIndex => {
                let index_value = self.get_a();
                let old_name_ptr = self
                    .var_path_stack
                    .pop_back()
                    .expect("Should have name_ptr");
                self.var_path_stack
                    .push_back(old_name_ptr.append_array_element(index_value));
            }
            Instruction::VarPathProperty(property_name) => {
                let old_name_ptr = self
                    .var_path_stack
                    .pop_back()
                    .expect("Should have name_ptr");
                self.var_path_stack.push_back(Path::Property(
                    Box::new(old_name_ptr),
                    property_name.clone(),
                ));
            }
            Instruction::CopyAToVarPath => {
                // get value to copy into name_ptr
                let a = self.get_a();
                // copy
                let v = self.resolve_name_ptr_mut().with_err_at(pos)?;
                *v = a;
            }
            Instruction::CopyVarPathToA => {
                let v = self.resolve_name_ptr_mut().with_err_at(pos)?;
                let v_copy = v.clone();
                self.set_a(v_copy);
            }
        }
        Ok(())
    }

    fn resolve_name_ptr_mut(&mut self) -> Result<&mut Variant, QError> {
        match self.var_path_stack.pop_back() {
            Some(n) => self.resolve_some_name_ptr_mut(n),
            _ => panic!("Root name_ptr was None"),
        }
    }

    fn resolve_some_name_ptr_mut(&mut self, name_ptr: Path) -> Result<&mut Variant, QError> {
        match name_ptr {
            Path::Root(var_name) => Ok(self.context_mut().get_or_create(var_name)),
            Path::ArrayElement(parent_name_ptr, indices) => {
                let parent_variant = self.resolve_some_name_ptr_mut(*parent_name_ptr)?;
                Self::resolve_array_mut(parent_variant, indices)
            }
            Path::Property(parent_name_ptr, property_name) => {
                let parent_variant = self.resolve_some_name_ptr_mut(*parent_name_ptr)?;
                Ok(Self::resolve_property_mut(parent_variant, &property_name))
            }
        }
    }

    fn resolve_array_mut(v: &mut Variant, indices: Vec<Variant>) -> Result<&mut Variant, QError> {
        match v {
            Variant::VArray(v_array) => {
                let int_indices: Result<Vec<i32>, QError> =
                    indices.into_iter().map(|v| i32::try_from(v)).collect();
                v_array.get_element_mut(int_indices?)
            }
            _ => panic!("Expected array, found {:?}", v),
        }
    }

    fn resolve_property_mut<'a>(v: &'a mut Variant, property_name: &BareName) -> &'a mut Variant {
        match v {
            Variant::VUserDefined(boxed_user_defined_value) => boxed_user_defined_value
                .get_mut(property_name)
                .expect("Property not defined, linter should have caught this"),
            _ => panic!("Expected user defined type, found {:?}", v),
        }
    }

    pub fn interpret(&mut self, instructions: Vec<InstructionNode>) -> Result<(), QErrorNode> {
        let mut i: usize = 0;
        let mut error_handler: Option<usize> = None;
        let mut exit: bool = false;
        while i < instructions.len() && !exit {
            let instruction = instructions[i].as_ref();
            let pos = instructions[i].pos();
            match self.interpret_one(&mut i, instruction, pos, &mut error_handler, &mut exit) {
                Ok(_) => {
                    i += 1;
                }
                Err(e) => match error_handler {
                    Some(error_idx) => {
                        i = error_idx;
                    }
                    None => {
                        return Err(e.patch_stacktrace(&self.stacktrace));
                    }
                },
            }
        }
        Ok(())
    }

    /// Takes the current context out of the interpreter.
    /// The interpreter is left with a dummy context.
    fn take_context(&mut self) -> Context {
        let dummy = Context::new(std::rc::Rc::clone(&self.user_defined_types));
        std::mem::replace(&mut self.context, dummy)
    }

    fn set_context(&mut self, context: Context) {
        self.context = context;
    }

    fn push_context(&mut self) {
        let current_context = self.take_context();
        self.set_context(current_context.push());
    }

    fn pop_context(&mut self) {
        let current_context = self.take_context();
        self.set_context(current_context.pop());
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
