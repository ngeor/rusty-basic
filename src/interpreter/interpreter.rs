use crate::common::*;
use crate::instruction_generator::{Instruction, InstructionNode};
use crate::interpreter::built_ins;
use crate::interpreter::context::*;
use crate::interpreter::context_owner::ContextOwner;
use crate::interpreter::io::FileManager;
use crate::interpreter::Stdlib;
use crate::linter::casting::cast;
use crate::linter::ResolvedUserDefinedTypes;
use crate::parser::TypeQualifier;
use crate::variant::{DefaultForTypes, Variant};
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

#[derive(Debug)]
pub struct Interpreter<S: Stdlib> {
    pub stdlib: S,
    pub context: Option<Context>,
    register_stack: RegisterStack,
    return_stack: Vec<usize>,
    stacktrace: Vec<Location>,
    pub function_result: Variant,
    pub file_manager: FileManager,
    pub user_defined_types: Rc<ResolvedUserDefinedTypes>,
}

impl<TStdlib: Stdlib> Interpreter<TStdlib> {
    pub fn new(stdlib: TStdlib, user_defined_types: ResolvedUserDefinedTypes) -> Self {
        let rc_user_defined_types = Rc::new(user_defined_types);
        let mut result = Interpreter {
            stdlib,
            context: Some(Context::new(Rc::clone(&rc_user_defined_types))),
            return_stack: vec![],
            register_stack: VecDeque::new(),
            stacktrace: vec![],
            function_result: Variant::VInteger(0),
            file_manager: FileManager::new(),
            user_defined_types: Rc::clone(&rc_user_defined_types),
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
                let casted = crate::linter::casting::cast(v, *q).with_err_at(pos)?;
                self.set_a(casted);
            }
            Instruction::Truncate(l) => {
                let v = self.get_a();
                let casted = crate::linter::casting::cast(v, TypeQualifier::DollarString)
                    .with_err_at(pos)?;
                self.set_a(match casted {
                    Variant::VString(s) => {
                        let len: usize = *l as usize;
                        if len < s.len() {
                            let chars: Vec<char> = s.chars().collect();
                            let c = &chars[0..len];
                            let s2: String = c.iter().collect();
                            Variant::VString(s2)
                        } else {
                            Variant::VString(s)
                        }
                    }
                    _ => casted,
                });
            }
            Instruction::Dim(resolved_declared_name) => {
                let v = Variant::default_variant_types(
                    &resolved_declared_name.type_definition(),
                    self.user_defined_types.as_ref(),
                );
                self.context_mut()
                    .set_variable(resolved_declared_name.clone(), v);
            }
            Instruction::Store(n) => {
                let v = self.get_a();
                self.context_mut().set_variable(n.clone(), v);
            }
            Instruction::StoreConst(n) => {
                let v = self.get_a();
                self.context_mut().set_constant(n.clone(), v);
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
            Instruction::CopyVarToA(n) => match self.context_ref().get_r_value(n) {
                Some(v) => self.set_a(v),
                None => panic!("CopyVarToA variable {:?} undefined at {:?}", n, pos),
            },
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
                let a = cast(self.get_a(), TypeQualifier::PercentInteger).with_err_at(pos)?;
                let b = cast(self.get_b(), TypeQualifier::PercentInteger).with_err_at(pos)?;
                self.set_a(a.and(b).with_err_at(pos)?);
            }
            Instruction::Or => {
                let a = cast(self.get_a(), TypeQualifier::PercentInteger).with_err_at(pos)?;
                let b = cast(self.get_b(), TypeQualifier::PercentInteger).with_err_at(pos)?;
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
                    .push_back_unnamed_ref_parameter(name.clone());
            }
            Instruction::PushUnnamedValParam => {
                let v = self.get_a();
                self.context_mut()
                    .demand_args()
                    .push_back_unnamed_val_parameter(v);
            }
            Instruction::SetNamedRefParam(named_ref_param) => {
                self.context_mut()
                    .demand_args()
                    .set_named_ref_parameter(named_ref_param);
            }
            Instruction::SetNamedValParam(param_q_name) => {
                let v = self.get_a();
                self.context_mut()
                    .demand_args()
                    .set_named_val_parameter(param_q_name, v);
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
            Instruction::StoreAToResult => {
                let v = self.get_a();
                self.function_result = v;
            }
            Instruction::CopyResultToA => {
                let v = self.function_result.clone();
                self.set_a(v);
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

    // shortcuts to common context_mut operations

    /// Pops the next unnamed argument, starting from the beginning.
    pub fn pop_unnamed_arg(&mut self) -> Option<Argument> {
        self.context_mut().demand_sub().pop_unnamed_arg()
    }

    /// Pops the value of the next unnamed argument, starting from the beginning.
    pub fn pop_unnamed_val(&mut self) -> Option<Variant> {
        self.context_mut().demand_sub().pop_unnamed_val()
    }

    pub fn pop_string(&mut self) -> String {
        self.pop_unnamed_val().unwrap().demand_string()
    }

    pub fn pop_integer(&mut self) -> i32 {
        self.pop_unnamed_val().unwrap().demand_integer()
    }

    pub fn pop_file_handle(&mut self) -> Result<FileHandle, QError> {
        match self.pop_unnamed_val().unwrap() {
            Variant::VFileHandle(f) => Ok(f),
            Variant::VInteger(i) => {
                if i >= 1 && i <= 255 {
                    Ok((i as u8).into())
                } else {
                    Err(QError::BadFileNameOrNumber)
                }
            }
            _ => Err(QError::TypeMismatch),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::assert_err;
    use crate::assert_has_variable;
    use crate::assert_prints;
    use crate::interpreter::context_owner::ContextOwner;
    use crate::interpreter::test_utils::*;
    use crate::linter::*;
    use crate::variant::Variant;

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

    mod assignment {
        use super::*;

        macro_rules! assert_assign_ok {
            ($program:expr, $expected_variable_name:expr, $expected_value:expr) => {
                let interpreter = interpret($program);
                let resolved_declared_name = ResolvedDeclaredName::parse($expected_variable_name);
                assert_eq!(
                    interpreter
                        .context_ref()
                        .get_r_value(&resolved_declared_name)
                        .unwrap(),
                    Variant::from($expected_value)
                );
            };
        }

        #[test]
        fn test_assign_literal_to_unqualified_float() {
            assert_assign_ok!("X = 1.0", "X!", 1.0_f32);
            assert_assign_ok!("X = -1.0", "X!", -1.0_f32);
            assert_assign_ok!("X = .5", "X!", 0.5_f32);
            assert_assign_ok!("X = -.5", "X!", -0.5_f32);
            assert_assign_ok!("X = 1", "X!", 1.0_f32);
            assert_assign_ok!("X = 3.14#", "X!", 3.14_f32);
        }

        #[test]
        fn test_assign_plus_expression_to_unqualified_float() {
            assert_assign_ok!("X = .5 + .5", "X!", 1.0_f32);
        }

        #[test]
        fn test_assign_literal_to_qualified_float() {
            assert_assign_ok!("X! = 1.0", "X!", 1.0_f32);
            assert_assign_ok!("X! = 1", "X!", 1.0_f32);
        }

        #[test]
        fn test_assign_literal_to_qualified_double() {
            assert_assign_ok!("X# = 1.0", "X#", 1.0_f64);
            assert_assign_ok!("X# = 1", "X#", 1.0_f64);
            assert_assign_ok!("X# = 3.14#", "X#", 3.14_f64);
        }

        #[test]
        fn test_assign_literal_to_qualified_string() {
            assert_assign_ok!("A$ = \"hello\"", "A$", "hello");
        }

        #[test]
        fn test_assign_literal_to_qualified_integer() {
            assert_assign_ok!("X% = 1.0", "X%", 1);
            assert_assign_ok!("X% = 1.1", "X%", 1);
            assert_assign_ok!("X% = 1.5", "X%", 2);
            assert_assign_ok!("X% = 1.9", "X%", 2);
            assert_assign_ok!("X% = 1", "X%", 1);
            assert_assign_ok!("X% = -1", "X%", -1);
            assert_assign_ok!("X% = 3.14#", "X%", 3);
        }

        #[test]
        fn test_assign_literal_to_qualified_long() {
            assert_assign_ok!("X& = 1.0", "X&", 1_i64);
            assert_assign_ok!("X& = 1.1", "X&", 1_i64);
            assert_assign_ok!("X& = 1.5", "X&", 2_i64);
            assert_assign_ok!("X& = 1.9", "X&", 2_i64);
            assert_assign_ok!("X& = 1", "X&", 1_i64);
            assert_assign_ok!("X& = -1", "X&", -1_i64);
            assert_assign_ok!("X& = 3.14#", "X&", 3_i64);
        }

        #[test]
        fn test_assign_same_variable_name_different_qualifiers() {
            let input = "A = 0.1
            A# = 3.14
            A$ = \"Hello\"
            A% = 1
            A& = 100";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 0.1_f32);
            assert_has_variable!(interpreter, "A#", 3.14);
            assert_has_variable!(interpreter, "A$", "Hello");
            assert_has_variable!(interpreter, "A%", 1);
            assert_has_variable!(interpreter, "A&", 100_i64);
        }

        #[test]
        fn test_assign_negated_variable() {
            let input = "A = -42
            B = -A";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", -42.0_f32);
            assert_has_variable!(interpreter, "B!", 42.0_f32);
        }

        #[test]
        fn test_assign_variable_bare_lower_case() {
            let input = "
            A = 42
            b = 12
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 42.0_f32);
            assert_has_variable!(interpreter, "a!", 42.0_f32);
            assert_has_variable!(interpreter, "B!", 12.0_f32);
            assert_has_variable!(interpreter, "b!", 12.0_f32);
        }

        #[test]
        fn test_assign_variable_typed_lower_case() {
            let input = "
            A% = 42
            b% = 12
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A%", 42);
            assert_has_variable!(interpreter, "a%", 42);
            assert_has_variable!(interpreter, "B%", 12);
            assert_has_variable!(interpreter, "b%", 12);
        }

        #[test]
        fn test_increment_variable_bare_lower_case() {
            let input = "
            A = 42
            A = a + 1
            b = 12
            B = b + 1
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 43_f32);
            assert_has_variable!(interpreter, "a!", 43_f32);
            assert_has_variable!(interpreter, "B!", 13_f32);
            assert_has_variable!(interpreter, "b!", 13_f32);
        }

        #[test]
        fn test_increment_variable_typed_lower_case() {
            let input = "
            A% = 42
            A% = a% + 1
            b% = 12
            B% = b% + 1
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A%", 43);
            assert_has_variable!(interpreter, "a%", 43);
            assert_has_variable!(interpreter, "B%", 13);
            assert_has_variable!(interpreter, "b%", 13);
        }

        #[test]
        fn test_assign_with_def_dbl() {
            let input = "
            DEFDBL A-Z
            A = 6.28
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A#", 6.28_f64);
        }

        #[test]
        fn test_assign_with_def_int() {
            let input = "
            DEFINT A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A%", 42);
        }

        #[test]
        fn test_assign_with_def_lng() {
            let input = "
            DEFLNG A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A&", 42_i64);
        }

        #[test]
        fn test_assign_with_def_sng() {
            let input = "
            DEFSNG A-Z
            A = 42
            A! = 3.14
            ";
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
        }

        #[test]
        fn test_assign_with_def_str() {
            let input = r#"
            DEFSTR A-Z
            A = "hello"
            A! = 3.14
            "#;
            let interpreter = interpret(input);
            assert_has_variable!(interpreter, "A!", 3.14_f32);
            assert_has_variable!(interpreter, "A$", "hello");
        }

        #[test]
        fn test_assign_integer_overflow() {
            assert_assign_ok!("A% = 32767", "A%", 32767_i32);
            assert_err!("A% = 32768", "Overflow", 1, 6);
            assert_assign_ok!("A% = -32768", "A%", -32768_i32);
            assert_err!("A% = -32769", "Overflow", 1, 6);
        }

        #[test]
        fn test_assign_long_overflow_ok() {
            assert_assign_ok!("A& = 2147483647", "A&", 2147483647_i64);
            assert_assign_ok!("A& = -2147483648", "A&", -2147483648_i64);
        }

        #[test]
        fn test_assign_long_overflow_err() {
            assert_err!("A& = 2147483648", "Overflow", 1, 6);
            assert_err!("A& = -2147483649", "Overflow", 1, 6);
        }

        #[test]
        fn test_same_variable_name_different_qualifiers() {
            let program = r#"
            A$ = "hello"
            A% = 42
            PRINT A$
            PRINT A%
            "#;
            assert_prints!(program, "hello", "42");
        }

        #[test]
        fn test_can_assign_to_parameter_hiding_name_of_function() {
            let program = r#"
            Hello 41
            FUNCTION Foo
            END FUNCTION

            SUB Hello(Foo)
            Foo = Foo + 1
            PRINT Foo
            END SUB
            "#;
            assert_prints!(program, "42");
        }
    }

    mod dim {
        use super::*;

        #[test]
        fn test_dim_string() {
            let program = r#"
            DIM A AS STRING
            A = "hello"
            PRINT A
            "#;
            assert_prints!(program, "hello");
        }

        #[test]
        fn test_dim_implicit_multiple_types_one_dim_one_assignment() {
            let program = r#"
            DIM A$
            A% = 42
            A$ = "hello"
            PRINT A$
            PRINT A%
            "#;
            assert_prints!(program, "hello", "42");
        }

        #[test]
        fn test_dim_implicit_multiple_types_two_dims() {
            let program = r#"
            DIM A$
            DIM A%
            A% = 42
            A$ = "hello"
            PRINT A$
            PRINT A%
            "#;
            assert_prints!(program, "hello", "42");
        }
    }

    mod function_implementation {
        use super::*;

        #[test]
        fn test_function_param_same_as_function_name_allowed() {
            let program = r#"
            PRINT Adding(41)
            FUNCTION Adding(Adding)
            Adding = Adding + 1
            END FUNCTION
            "#;
            assert_prints!(program, "42");
        }

        #[test]
        fn test_function_param_same_as_function_name_compact_single_allowed() {
            let program = r#"
            PRINT Adding(41)
            FUNCTION Adding(Adding!)
            Adding = Adding + 1
            END FUNCTION
            "#;
            assert_prints!(program, "42");
        }

        #[test]
        fn test_function_param_same_as_other_function_allowed() {
            let program = r#"
            PRINT Bar(2)

            FUNCTION Foo(Foo)
                Foo = Foo + 1
            END FUNCTION

            FUNCTION Bar(Foo)
                Bar = Foo + Foo(Foo)
            END FUNCTION
            "#;
            assert_prints!(program, "5");
        }
    }

    mod sub_implementation {
        use super::*;

        #[test]
        fn test_sub_params_same_name_different_qualifier() {
            let program = r#"
            Hello 42, "answer"
            SUB Hello(A%, A$)
                PRINT A%
                PRINT A$
            END SUB
            "#;
            assert_prints!(program, "42", "answer");
        }

        #[test]
        fn test_sub_param_expression_different_qualifier() {
            let program = r#"
            Hello "answer"
            SUB Hello(A$)
                A% = 42
                PRINT A%
            END SUB
            "#;
            assert_prints!(program, "42");
        }

        #[test]
        fn test_sub_param_same_as_other_function_allowed() {
            let program = r#"
            Hello 2
            SUB Hello(Foo)
                PRINT Foo + Foo(Foo)
            END SUB
            FUNCTION Foo(Foo)
                Foo = Foo + 1
            END FUNCTION
            "#;
            assert_prints!(program, "5");
        }
    }

    mod user_defined_type {
        use super::*;

        #[test]
        fn test_user_defined_type_card() {
            let input = r#"
            TYPE Card
                Suit AS STRING * 9
                Value AS INTEGER
            END TYPE

            DIM C AS Card
            C.Suit = "Hearts"
            C.Value = 1

            PRINT C.Suit
            PRINT C.Value
            "#;

            assert_prints!(input, "Hearts", "1");
        }

        #[test]
        fn test_user_defined_type_nested() {
            let input = r#"
            TYPE PostCode
                Prefix AS INTEGER
                Suffix AS STRING * 2
            END TYPE

            TYPE Address
                Street AS STRING * 100
                PostCode AS PostCode
            END TYPE

            DIM A AS Address
            A.PostCode.Prefix = 1234
            A.PostCode.Suffix =  "CZ"

            PRINT A.PostCode.Prefix
            PRINT A.PostCode.Suffix
            "#;

            assert_prints!(input, "1234", "CZ");
        }

        #[test]
        fn test_truncate_string_at_declared_length() {
            let input = r#"
            TYPE Address ' A basic address type
                ' post code
                PostCode AS STRING * 6 ' comment here
                ' comment here too
            END TYPE

            DIM a AS Address
            a.PostCode = "1234 AZ"
            PRINT a.PostCode
            "#;
            assert_prints!(input, "1234 A");
        }

        #[test]
        fn test_string_const_length() {
            let input = r#"
            CONST L = 6
            TYPE Address
                PostCode AS STRING * L
            END TYPE

            DIM a AS Address
            a.PostCode = "1234 AZ"
            PRINT a.PostCode
            "#;
            assert_prints!(input, "1234 A");
        }

        #[test]
        fn test_assign() {
            let input = r#"
            TYPE Address
                Street AS STRING * 50
                HouseNumber AS INTEGER
            END TYPE
            DIM a AS Address
            DIM b AS Address
            a.Street = "Hello"
            a.HouseNumber = 42
            b = a
            PRINT b.Street
            PRINT b.HouseNumber
            "#;
            assert_prints!(input, "Hello", "42");
        }

        #[test]
        fn test_assign_is_by_val() {
            let input = r#"
            TYPE Address
                Street AS STRING * 50
                HouseNumber AS INTEGER
            END TYPE
            DIM a AS Address
            DIM b AS Address

            a.Street = "original value"
            a.HouseNumber = 42

            b = a
            b.Street = "modified value"

            PRINT a.Street
            PRINT b.Street
            "#;
            assert_prints!(input, "original value", "modified value");
        }

        #[test]
        fn test_modify_in_sub() {
            let input = r#"
            TYPE LPARAM
                LoWord AS INTEGER
                HiWord AS INTEGER
            END TYPE

            DECLARE SUB Swap.LParam(x AS LPARAM)

            DIM p AS LPARAM
            p.LoWord = 1
            p.HiWord = 2
            Swap.LParam p
            PRINT p.LoWord
            PRINT p.HiWord

            SUB Swap.LParam(x AS LPARAM)
                p = x.LoWord
                x.LoWord = x.HiWord
                x.HiWord = p
            END SUB
            "#;
            assert_prints!(input, "2", "1");
        }

        // TODO concat two string * 9 together, what happens to the result if assigned to regular string
    }
}
