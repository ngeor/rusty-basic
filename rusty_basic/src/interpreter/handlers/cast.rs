use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::interpreter::string_utils::fix_length;
use rusty_common::QError;
use rusty_linter::CastVariant;
use rusty_parser::TypeQualifier;
use rusty_variant::Variant;

pub fn cast<T: InterpreterTrait>(interpreter: &mut T, q: &TypeQualifier) -> Result<(), QError> {
    let v = interpreter.registers().get_a();
    let casted = v.cast(*q)?;
    interpreter.registers_mut().set_a(casted);
    Ok(())
}

pub fn fix_length_in_a<T: InterpreterTrait>(interpreter: &mut T, l: &u16) -> Result<(), QError> {
    let v = interpreter.registers().get_a();
    let mut casted = v.cast(TypeQualifier::DollarString)?;
    if let Variant::VString(s) = &mut casted {
        let len: usize = *l as usize;
        fix_length(s, len);
    }
    interpreter.registers_mut().set_a(casted);
    Ok(())
}
