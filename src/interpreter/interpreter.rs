use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionNode};
use crate::interpreter::context::*;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::{InterpreterError, Result, Stdlib};

use crate::variant::Variant;

use std::cmp::Ordering;
use std::collections::VecDeque;
use std::convert::TryFrom;

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

#[derive(Debug)]
pub struct Interpreter<S: Stdlib> {
    pub stdlib: S,
    pub context: Option<Context>,
    register_stack: RegisterStack,
    return_stack: Vec<usize>,
    stacktrace: Vec<Location>,
    pub function_result: Variant,
}

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib) -> Self {
        let mut result = Interpreter {
            stdlib,
            context: Some(Context::new()),
            return_stack: vec![],
            register_stack: VecDeque::new(),
            stacktrace: vec![],
            function_result: Variant::VInteger(0),
        };
        result.register_stack.push_back(Registers::new());
        result
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
    ) -> Result<()> {
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
            Instruction::Store(n) => {
                let v = self.get_a();
                self.context_mut()
                    .set_variable(n.clone(), v)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
            }
            Instruction::StoreConst(n) => {
                let v = self.get_a();
                self.context_mut()
                    .set_constant(n.clone(), v)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
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
                let c = a
                    .plus(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                self.set_a(c);
            }
            Instruction::Minus => {
                let a = self.get_a();
                let b = self.get_b();
                let c = a
                    .minus(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                self.set_a(c);
            }
            Instruction::NegateA => {
                let a = self.get_a();
                let c = a
                    .negate()
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                self.set_a(c);
            }
            Instruction::NotA => {
                let a = self.get_a();
                let c = a
                    .unary_not()
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                self.set_a(c);
            }
            Instruction::CopyVarToA(n) => {
                let name_node = n.clone().at(pos);
                match self.context_ref().get_r_value(name_node.as_ref()) {
                    Some(v) => self.set_a(v),
                    None => panic!("Variable {:?} undefined at {:?}", n, pos),
                }
            }
            Instruction::Equal => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a
                    .cmp(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                let is_true = order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::Less => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a
                    .cmp(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                let is_true = order == Ordering::Less;
                self.set_a(is_true.into());
            }
            Instruction::Greater => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a
                    .cmp(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                let is_true = order == Ordering::Greater;
                self.set_a(is_true.into());
            }
            Instruction::LessOrEqual => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a
                    .cmp(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                let is_true = order == Ordering::Less || order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::GreaterOrEqual => {
                let a = self.get_a();
                let b = self.get_b();
                let order = a
                    .cmp(&b)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                let is_true = order == Ordering::Greater || order == Ordering::Equal;
                self.set_a(is_true.into());
            }
            Instruction::JumpIfFalse(resolved_idx) => {
                let a = self.get_a();
                let is_true: bool =
                    bool::try_from(a).map_err(|e| InterpreterError::new_with_pos(e, pos))?;
                if !is_true {
                    *i = resolved_idx - 1; // the +1 will happen at the end of the loop
                }
            }
            Instruction::Jump(resolved_idx) => {
                *i = resolved_idx - 1;
            }
            Instruction::PreparePush => {
                self.push_args_context();
            }
            Instruction::PushStack => {
                self.swap_args_with_sub_context();
                self.stacktrace.insert(0, pos);
            }
            Instruction::PopStack => {
                self.pop();
                self.stacktrace.remove(0);
            }
            Instruction::PushUnnamedRefParam(name) => {
                self.context_mut()
                    .demand_args()
                    .push_back_unnamed_ref_parameter(name.clone())
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
            }
            Instruction::PushUnnamedValParam => {
                let v = self.get_a();

                self.context_mut()
                    .demand_args()
                    .push_back_unnamed_val_parameter(v)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
            }
            Instruction::SetNamedRefParam(named_ref_param) => {
                self.context_mut()
                    .demand_args()
                    .set_named_ref_parameter(named_ref_param)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
            }
            Instruction::SetNamedValParam(param_q_name) => {
                let v = self.get_a();

                self.context_mut()
                    .demand_args()
                    .set_named_val_parameter(param_q_name, v)
                    .map_err(|e| InterpreterError::new_with_pos(e, pos))?;
            }
            Instruction::BuiltInSub(n) => {
                self.run_built_in_sub(n, pos)?;
            }
            Instruction::BuiltInFunction(n) => {
                self.run_built_in_function(n, pos)?;
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
            Instruction::StoreAToResult => {
                let v = self.get_a();
                self.function_result = v;
            }
            Instruction::CopyResultToA => {
                let v = self.function_result.clone();
                self.set_a(v);
            }
            Instruction::Throw(msg) => {
                self.throw(msg, pos)?;
            }
        }
        Ok(())
    }

    pub fn interpret(&mut self, instructions: Vec<InstructionNode>) -> Result<()> {
        let mut i: usize = 0;
        let mut error_handler: Option<usize> = None;
        let mut exit: bool = false;
        while i < instructions.len() && !exit {
            let instruction = instructions[i].as_ref();
            let pos = instructions[i].location();
            match self.interpret_one(&mut i, instruction, pos, &mut error_handler, &mut exit) {
                Ok(_) => {
                    i += 1;
                }
                Err(e) => match error_handler {
                    Some(error_idx) => {
                        i = error_idx;
                    }
                    None => {
                        return Err(e.with_existing_stacktrace(&self.stacktrace));
                    }
                },
            }
        }
        Ok(())
    }

    fn throw(&mut self, msg: &String, pos: Location) -> Result<()> {
        Err(InterpreterError::new_with_pos(msg, pos))
    }
}

#[cfg(test)]
mod tests {
    use super::super::test_utils::*;

    #[test]
    fn test_interpret_print_hello_world_one_arg() {
        let input = "PRINT \"Hello, world!\"";
        assert_eq!(interpret(input).stdlib.output, vec!["Hello, world!"]);
    }

    #[test]
    fn test_interpret_print_hello_world_two_args() {
        let input = r#"PRINT "Hello", "world!""#;
        assert_eq!(interpret(input).stdlib.output, vec!["Hello world!"]);
    }

    #[test]
    fn test_interpret_print_hello_world_two_args_one_is_function() {
        let input = r#"
        PRINT "Hello", Test(1)
        FUNCTION Test(N)
            Test = N + 1
        END FUNCTION
        "#;
        assert_eq!(interpret(input).stdlib.output, vec!["Hello 2"]);
    }

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
                "Fibonacci of 0 is 0",
                "Fibonacci of 1 is 1",
                "Fibonacci of 2 is 1",
                "Fibonacci of 3 is 2",
                "Fibonacci of 4 is 3",
                "Fibonacci of 5 is 5",
                "Fibonacci of 6 is 8",
                "Fibonacci of 7 is 13",
                "Fibonacci of 8 is 21",
                "Fibonacci of 9 is 34",
                "Fibonacci of 10 is 55"
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
