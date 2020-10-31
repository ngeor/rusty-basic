use crate::common::QError;
use crate::interpreter::interpreter_trait::InterpreterTrait;
use crate::linter::ExpressionType;
use crate::parser::{BareName, TypeQualifier};
use crate::variant::{UserDefinedTypeValue, VArray, Variant};
use std::convert::TryFrom;

pub fn allocate_built_in<T: InterpreterTrait>(
    interpreter: &mut T,
    q: TypeQualifier,
) -> Result<(), QError> {
    let v = Variant::from(q);
    interpreter.registers_mut().set_a(v);
    Ok(())
}

pub fn allocate_fixed_length_string<T: InterpreterTrait>(
    interpreter: &mut T,
    len: u16,
) -> Result<(), QError> {
    let s: String = (0..len).map(|_| ' ').collect();
    let v = Variant::VFixedLengthString(s);
    interpreter.registers_mut().set_a(v);
    Ok(())
}

pub fn allocate_array<T: InterpreterTrait>(
    interpreter: &mut T,
    element_type: &ExpressionType,
) -> Result<(), QError> {
    let r_args: Result<Vec<i32>, QError> = interpreter
        .context_mut()
        .arguments_stack()
        .pop()
        .into_iter()
        .map(|(_, v)| v)
        .map(|v| i32::try_from(v))
        .collect();
    let args = r_args?;
    let mut dimensions: Vec<(i32, i32)> = vec![];
    let mut i: usize = 0;
    let mut elements: Vec<Variant> = vec![];
    while i < args.len() {
        let lbound = args[i];
        i += 1;
        let ubound = args[i];
        if ubound < lbound {
            return Err(QError::SubscriptOutOfRange);
        }
        i += 1;
        dimensions.push((lbound, ubound));
        for _i in lbound..ubound + 1 {
            elements.push(element_type.default_variant(interpreter.user_defined_types()));
        }
    }
    let array = Variant::VArray(Box::new(VArray {
        dimensions,
        elements,
    }));
    interpreter.registers_mut().set_a(array);
    Ok(())
}

pub fn allocate_user_defined_type<T: InterpreterTrait>(
    interpreter: &mut T,
    user_defined_type_name: &BareName,
) -> Result<(), QError> {
    let v: Variant = Variant::VUserDefined(Box::new(UserDefinedTypeValue::new(
        user_defined_type_name,
        interpreter.user_defined_types(),
    )));
    interpreter.registers_mut().set_a(v);
    Ok(())
}
