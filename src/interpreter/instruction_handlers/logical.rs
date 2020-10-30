use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::parser::TypeQualifier;

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
