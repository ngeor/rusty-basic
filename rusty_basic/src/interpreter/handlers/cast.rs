use crate::interpreter::interpreter_trait::InterpreterTrait;
use rusty_common::{QError, StringUtils};
use rusty_parser::variant::Variant;
use rusty_parser::TypeQualifier;

pub fn cast<T: InterpreterTrait>(interpreter: &mut T, q: &TypeQualifier) -> Result<(), QError> {
    let v = interpreter.registers().get_a();
    let casted = v.cast(*q)?;
    interpreter.registers_mut().set_a(casted);
    Ok(())
}

pub fn fix_length<T: InterpreterTrait>(interpreter: &mut T, l: &u16) -> Result<(), QError> {
    let v = interpreter.registers().get_a();
    let casted = v.cast(TypeQualifier::DollarString)?;
    interpreter.registers_mut().set_a(match casted {
        Variant::VString(s) => {
            let len: usize = *l as usize;
            Variant::VString(s.fix_length(len))
        }
        _ => casted,
    });
    Ok(())
}
