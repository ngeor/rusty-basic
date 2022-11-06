use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::RuntimeError;
use std::cmp::Ordering;

pub fn equal<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| order == Ordering::Equal)
}

pub fn not_equal<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| order != Ordering::Equal)
}

pub fn less<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| order == Ordering::Less)
}

pub fn greater<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| order == Ordering::Greater)
}

pub fn less_or_equal<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| {
        order == Ordering::Less || order == Ordering::Equal
    })
}

pub fn greater_or_equal<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), RuntimeError> {
    cmp(interpreter, |order| {
        order == Ordering::Greater || order == Ordering::Equal
    })
}

fn cmp<T: InterpreterTrait, F: FnOnce(Ordering) -> bool>(
    interpreter: &mut T,
    predicate: F,
) -> Result<(), RuntimeError> {
    let a = interpreter.registers().get_a();
    let b = interpreter.registers().get_b();
    let order = a.try_cmp(&b)?;
    let is_true = predicate(order);
    interpreter.registers_mut().set_a(is_true.into());
    Ok(())
}
