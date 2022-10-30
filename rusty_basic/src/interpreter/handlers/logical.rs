use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::*;
use rusty_parser::TypeQualifier;

pub fn and<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let a = interpreter
        .registers()
        .get_a()
        .cast(TypeQualifier::PercentInteger)?;
    let b = interpreter
        .registers()
        .get_b()
        .cast(TypeQualifier::PercentInteger)?;
    interpreter.registers_mut().set_a(a.and(b)?);
    Ok(())
}

pub fn or<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let a = interpreter
        .registers()
        .get_a()
        .cast(TypeQualifier::PercentInteger)?;
    let b = interpreter
        .registers()
        .get_b()
        .cast(TypeQualifier::PercentInteger)?;
    interpreter.registers_mut().set_a(a.or(b)?);
    Ok(())
}

pub fn negate_a<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let a = interpreter.registers().get_a();
    let n = a.negate()?;
    interpreter.registers_mut().set_a(n);
    Ok(())
}

pub fn not_a<T: InterpreterTrait>(interpreter: &mut T) -> Result<(), QError> {
    let a = interpreter.registers().get_a();
    let n = a.unary_not()?;
    interpreter.registers_mut().set_a(n);
    Ok(())
}
