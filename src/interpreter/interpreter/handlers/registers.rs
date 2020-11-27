use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::registers::Registers;
use crate::variant::Variant;

pub fn push_registers<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.register_stack().push(Registers::new());
}

pub fn pop_registers<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.register_stack().pop();
}

pub fn load_into_a<T: InterpreterTrait>(interpreter: &mut T, v: &Variant) {
    interpreter.registers_mut().set_a(v.clone());
}

pub fn copy_a_to_b<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_a_to_b();
}

pub fn copy_a_to_c<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_a_to_c();
}

pub fn copy_a_to_d<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_a_to_d();
}

pub fn copy_c_to_b<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_c_to_b();
}

pub fn copy_d_to_a<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_d_to_a();
}

pub fn copy_d_to_b<T: InterpreterTrait>(interpreter: &mut T) {
    interpreter.registers_mut().copy_d_to_b();
}
