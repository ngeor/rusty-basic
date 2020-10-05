use crate::built_ins::BuiltInFunction;
use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionNode};
use crate::interpreter::context::*;
use crate::interpreter::io::FileManager;
use crate::interpreter::Stdlib;
use crate::interpreter::{built_ins, Printer};
use crate::linter::{DimName, UserDefinedTypes};
use crate::parser::{QualifiedName, TypeQualifier};
use crate::variant::Variant;
use std::cmp::Ordering;
use std::collections::VecDeque;
use std::convert::TryFrom;
use std::rc::Rc;

#[derive(Debug)]
pub struct Registers {
    a: Variant,
    b: Variant,
    c: Variant,
    d: Variant,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            a: Variant::VInteger(0),
            b: Variant::VInteger(0),
            c: Variant::VInteger(0),
            d: Variant::VInteger(0),
        }
    }

    pub fn get_a(&self) -> Variant {
        self.a.clone()
    }

    pub fn get_b(&self) -> Variant {
        self.b.clone()
    }

    pub fn set_a(&mut self, v: Variant) {
        self.a = v;
    }

    pub fn copy_a_to_b(&mut self) {
        self.b = self.a.clone();
    }

    pub fn copy_a_to_c(&mut self) {
        self.c = self.a.clone();
    }

    pub fn copy_a_to_d(&mut self) {
        self.d = self.a.clone();
    }

    pub fn copy_c_to_b(&mut self) {
        self.b = self.c.clone();
    }

    pub fn copy_d_to_a(&mut self) {
        self.a = self.d.clone();
    }

    pub fn copy_d_to_b(&mut self) {
        self.b = self.d.clone();
    }

    pub fn swap_a_with_b(&mut self) {
        std::mem::swap(&mut self.a, &mut self.b);
    }
}

pub type RegisterStack = VecDeque<Registers>;

pub struct Interpreter<S: Stdlib> {
    pub stdlib: S,
    context: Context,
    register_stack: RegisterStack,
    return_stack: Vec<usize>,
    stacktrace: Vec<Location>,
    pub file_manager: FileManager,
    pub user_defined_types: Rc<UserDefinedTypes>,
}

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib, user_defined_types: UserDefinedTypes) -> Self {
        let rc_user_defined_types = Rc::new(user_defined_types);
        let mut result = Interpreter {
            stdlib,
            context: Context::new(Rc::clone(&rc_user_defined_types)),
            return_stack: vec![],
            register_stack: VecDeque::new(),
            stacktrace: vec![],
            file_manager: FileManager::new(),
            user_defined_types: Rc::clone(&rc_user_defined_types),
        };
        result.register_stack.push_back(Registers::new());
        result
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn context_mut(&mut self) -> &mut Context {
        &mut self.context
    }

    fn registers_ref(&self) -> &Registers {
        self.register_stack.back().unwrap()
    }

    fn registers_mut(&mut self) -> &mut Registers {
        self.register_stack.back_mut().unwrap()
    }

    fn get_a(&self) -> Variant {
        self.registers_ref().get_a()
    }

    fn get_b(&self) -> Variant {
        self.registers_ref().get_b()
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
            Instruction::Dim(dim_name) => {
                self.set_default_value(dim_name);
            }
            Instruction::Store(n) => {
                let v = self.get_a();
                self.context.set_variable(n.clone(), v);
            }
            Instruction::StoreConst(n) => {
                let v = self.get_a();
                self.context.set_constant(n.clone(), v);
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
                let a = self.get_a();
                let b = self.get_b();
                let c = a.plus(b).with_err_at(pos)?;
                self.set_a(c);
            }
            Instruction::Minus => {
                let a = self.get_a();
                let b = self.get_b();
                let c = a.minus(b).with_err_at(pos)?;
                self.set_a(c);
            }
            Instruction::Multiply => {
                let a = self.get_a();
                let b = self.get_b();
                let c = a.multiply(b).with_err_at(pos)?;
                self.set_a(c);
            }
            Instruction::Divide => {
                let a = self.get_a();
                let b = self.get_b();
                let c = a.divide(b).with_err_at(pos)?;
                self.set_a(c);
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
            Instruction::CopyVarToA(var_name) => {
                let v = match self.context.get_r_value(var_name) {
                    Some(v) => v.clone(),
                    None => self.set_default_value(var_name),
                };
                self.set_a(v);
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
                let a = self
                    .get_a()
                    .cast(TypeQualifier::PercentInteger)
                    .with_err_at(pos)?;
                let b = self
                    .get_b()
                    .cast(TypeQualifier::PercentInteger)
                    .with_err_at(pos)?;
                self.set_a(a.and(b).with_err_at(pos)?);
            }
            Instruction::Or => {
                let a = self
                    .get_a()
                    .cast(TypeQualifier::PercentInteger)
                    .with_err_at(pos)?;
                let b = self
                    .get_b()
                    .cast(TypeQualifier::PercentInteger)
                    .with_err_at(pos)?;
                self.set_a(a.or(b).with_err_at(pos)?);
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
            Instruction::PopStack(opt_function_name) => {
                // get the function result
                let function_result: Option<Variant> = match opt_function_name {
                    Some(function_name) => {
                        let r: DimName = function_name.clone().into();
                        match self.context.get_r_value(&r) {
                            Some(v) => Some(v.clone()),
                            None => {
                                // it was a function, but the implementation did not set a result
                                let QualifiedName { qualifier, .. } = function_name;
                                Some(Variant::from(qualifier))
                            }
                        }
                    }
                    None => None,
                };
                self.pop_context();
                self.stacktrace.remove(0);
                // store function result into A now that we're in the parent context
                match function_result {
                    Some(v) => {
                        self.set_a(v);
                    }
                    None => {}
                }
            }
            Instruction::CopyToParent(idx, parent_var_name) => {
                self.context.copy_to_parent(*idx, parent_var_name);
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
                self.return_stack.push(*addr);
            }
            Instruction::PopRet => {
                let addr = self.return_stack.pop().unwrap();
                *i = addr - 1;
            }
            Instruction::Throw(interpreter_error) => {
                return Err(interpreter_error.clone()).with_err_at(pos);
            }
        }
        Ok(())
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

    fn set_default_value(&mut self, dim_name: &DimName) -> Variant {
        let v = dim_name
            .dim_type()
            .default_variant(self.user_defined_types.as_ref());
        self.context.set_variable(dim_name.clone(), v.clone());
        v
    }
}

pub trait SetVariable<K, V> {
    fn set_variable(&mut self, name: K, value: V);
}

impl<S: Stdlib, V> SetVariable<BuiltInFunction, V> for Interpreter<S>
where
    Variant: From<V>,
{
    fn set_variable(&mut self, name: BuiltInFunction, value: V) {
        self.context.set_variable(name.into(), value.into());
    }
}

#[cfg(test)]
mod tests {
    use crate::interpreter::test_utils::*;

    #[test]
    fn test_interpreter_fixture_hello1() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO1.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello2() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO2.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_hello_s() {
        let stdlib = MockStdlib::new();
        interpret_file("HELLO_S.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_print_10() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_PRINT_10.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_for_nested() {
        let stdlib = MockStdlib::new();
        interpret_file("FOR_NESTED.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_fib_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("10");
        let interpreter = interpret_file("FIB.BAS", stdlib).unwrap();
        let output = interpreter.stdlib.output;
        assert_eq!(
            output,
            vec![
                "Enter the number of fibonacci to calculate",
                "Fibonacci of  0             is            0",
                "Fibonacci of  1             is            1",
                "Fibonacci of  2             is            1",
                "Fibonacci of  3             is            2",
                "Fibonacci of  4             is            3",
                "Fibonacci of  5             is            5",
                "Fibonacci of  6             is            8",
                "Fibonacci of  7             is            13",
                "Fibonacci of  8             is            21",
                "Fibonacci of  9             is            34",
                "Fibonacci of  10            is            55"
            ]
        );
    }

    #[test]
    fn test_interpreter_fixture_fib_fq_bas() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("11");
        interpret_file("FIB_FQ.BAS", stdlib).unwrap();
    }

    #[test]
    fn test_interpreter_fixture_input() {
        let mut stdlib = MockStdlib::new();
        stdlib.add_next_input("");
        interpret_file("INPUT.BAS", stdlib).unwrap();
    }
}
