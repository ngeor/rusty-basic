use rusty_variant::{Variant, VariantError};

use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;

pub fn plus<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    reduce_a_b_into_a(interpreter, |a, b| a.plus(b))
}

pub fn minus<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    reduce_a_b_into_a(interpreter, |a, b| a.minus(b))
}

pub fn multiply<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    reduce_a_b_into_a(interpreter, |a, b| a.multiply(b))
}

pub fn divide<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    reduce_a_b_into_a(interpreter, |a, b| a.divide(b))
}

pub fn modulo<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    reduce_a_b_into_a(interpreter, |a, b| a.modulo(b))
}

fn reduce_a_b_into_a<T: InterpreterTrait, F>(interpreter: &mut T, f: F) -> Result<(), RuntimeError>
where
    F: FnOnce(Variant, Variant) -> Result<Variant, VariantError>,
{
    let a = interpreter.registers().get_a();
    let b = interpreter.registers().get_b();
    let c = f(a, b)?;
    interpreter.registers_mut().set_a(c);
    Ok(())
}
